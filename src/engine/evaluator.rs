use crate::engine::convention::hgroup::h_group_core::count_bad_touches;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::decision_tree::Score;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// Per-term breakdown of a leaf evaluation score.
///
/// Each field is the weighted contribution of one scoring term; `total` is their sum.
/// Use `Display` for a compact human-readable representation.
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    /// `score_weight * game_score` — weighted Hanabi score achieved so far.
    pub game_score: f64,
    /// Strike penalty indexed by strike count; zero at 0 strikes, steep at 2.
    pub strike_penalty: f64,
    /// `pace_weight * pace` (clamped to −10 from below) — breathing room remaining.
    pub pace: f64,
    /// `efficiency_weight * required_efficiency` — remaining discard burden penalty.
    pub efficiency_penalty: f64,
    /// `critical_in_hand_weight * critical_cards_in_hand` — reward for keeping critical cards safe.
    pub critical_in_hand: f64,
    /// `lost_score_ceiling_weight * (theoretical_max − max_achievable)` — penalty for lost score ceiling.
    pub lost_ceiling_penalty: f64,
    /// `empathy_weight * empathy_precision` — reward for narrower identity ranges (disabled by default).
    pub empathy_bonus: f64,
    /// `clue_token_weight * whole_clue_tokens` — reward for clue tokens remaining.
    pub clue_tokens: f64,
    /// `known_playable_weight * known_playable_in_hands` — reward for cards known (by their owner) to be playable.
    pub known_playable: f64,
    /// `team_empathy_weight * team_empathy_score` — reward for fraction of identity uncertainty eliminated across all own-hand cards.
    pub team_empathy: f64,
    /// `resolved_cards_weight * resolved_count` — reward for cards fully resolved to a single identity.
    pub resolved_cards: f64,
    /// Sum of all terms above.
    pub total: f64,
}

impl std::fmt::Display for ScoreBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total={:.2} [score={:.1} -strike={:.1} +pace={:.1} -eff={:.1} +crit={:.1} -ceil={:.1} +emp={:.1} +clue={:.1} +play={:.1} +team_emp={:.1} +res={:.1}]",
            self.total,
            self.game_score,
            self.strike_penalty,
            self.pace,
            self.efficiency_penalty,
            self.critical_in_hand,
            self.lost_ceiling_penalty,
            self.empathy_bonus,
            self.clue_tokens,
            self.known_playable,
            self.team_empathy,
            self.resolved_cards,
        )
    }
}

/// Trait for scoring a leaf game state during search.
pub trait Evaluator: Send + Sync {
    fn score(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> Score;

    /// Per-term breakdown of the score. The default implementation returns only the total;
    /// override this to expose individual contributions for debugging.
    fn score_breakdown(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> ScoreBreakdown {
        ScoreBreakdown {
            game_score: 0.0,
            strike_penalty: 0.0,
            pace: 0.0,
            efficiency_penalty: 0.0,
            critical_in_hand: 0.0,
            lost_ceiling_penalty: 0.0,
            empathy_bonus: 0.0,
            clue_tokens: 0.0,
            known_playable: 0.0,
            team_empathy: 0.0,
            resolved_cards: 0.0,
            total: self.score(table_state, static_data, team_knowledge),
        }
    }

    /// Immediate bonus for a clue action based on empathy narrowing of the touched cards.
    /// `touched` are the deck indices touched by the clue; `receiver` is the clue target.
    fn clue_precision_bonus(
        &self,
        _touched: &[u8],
        _receiver: usize,
        _static_data: &StaticGameData,
        _team_knowledge: &TeamKnowledge,
        _table_state: &TableState,
    ) -> Score {
        0.0
    }
}

/// Default heuristic evaluator.
///
/// Scoring terms:
/// - `score_weight * game_score`                        — reward progress
/// - `-strike_penalty(strikes)`                         — steep penalty near 3 strikes
/// - `pace_weight * pace` (clamped)                     — reward breathing room
/// - `-efficiency_weight * required_efficiency`         — penalise remaining discard burden
/// - `critical_in_hand_weight * critical_in_hand`       — reward keeping critical cards safe
/// - `-lost_score_ceiling_weight * lost_score_ceiling`  — penalise any reduction in max achievable score
/// - `empathy_weight * empathy_precision`               — reward narrower inferred identity ranges on clued cards
///
/// Per-clue immediate adjustments (applied at the root, not propagated through the tree):
/// - `empathy_weight * resolved_touched_cards`          — precision bonus for clues that fully resolve touched cards
/// - `-good_touch_penalty * bad_touch_count`            — penalty for each touched card with no overlap with still-needed
///                                                        cards (good-touch principle violation)
pub struct DefaultEvaluator {
    /// Multiplier for the Hanabi game score term; dominates the total and keeps score progress as the primary objective.
    pub score_weight: f64,
    /// Per-strike penalty values indexed by strike count (0, 1, 2).
    /// Strike 3 is terminal and handled by the search, not the evaluator.
    pub strike_penalties: [f64; 3],
    /// Multiplier for the pace term (`pace = clues_left + cards_left − cards_needed`), clamped to −10 from below.
    pub pace_weight: f64,
    /// Multiplier for the required-efficiency penalty; higher values penalise states that demand many future discards.
    pub efficiency_weight: f64,
    /// Multiplier for the critical-cards-in-hand bonus; rewards positions where critical cards are unlikely to be discarded.
    pub critical_in_hand_weight: f64,
    /// Penalty per point of score ceiling lost (last copy discarded, or pace < 0).
    pub lost_score_ceiling_weight: f64,
    /// Reward for each bit of identity information gained on clued cards.
    /// For each clued card in a player's own hand, we add
    /// `empathy_weight * (max_identities - popcount(empathy))`.
    pub empathy_weight: f64,
    /// Penalty per touched card that has no overlap with still-needed cards
    /// (good-touch principle violation). Should be lower than `lost_score_ceiling_weight`
    /// so that ceiling preservation takes precedence.
    pub good_touch_penalty: f64,
    /// Reward per whole clue token remaining. Preserving clue tokens is valuable
    /// because they enable future saves and plays.
    pub clue_token_weight: f64,
    /// Immediate reward per touched card that is fully resolved to a single identity
    /// after the clue is applied (good-touch precision bonus). Distinct from
    /// `empathy_weight`, which applies to the leaf evaluation; this one fires only
    /// at the root for clue actions.
    pub clue_precision_weight: f64,
    /// Reward per card in any player's own hand where the entire empathy set is a subset
    /// of the currently playable cards (or the card carries a `Signal::Play`). Captures
    /// the value of clues that set up plays the search depth may not reach.
    pub known_playable_weight: f64,
    /// Reward proportional to the total fraction of identity uncertainty eliminated across
    /// all own-hand cards for all players: `Σ (max_ids − popcount) / max_ids`.
    /// Encourages states where the team knows more about every card, not just clued ones.
    pub team_empathy_weight: f64,
    /// Reward per card in any player's own hand that is fully resolved to a single identity
    /// (`popcount == 1`). Sharper signal than `team_empathy_weight`; rewards complete certainty.
    pub resolved_cards_weight: f64,
}

impl Default for DefaultEvaluator {
    fn default() -> Self {
        DefaultEvaluator {
            score_weight: 10.0_f64,
            strike_penalties: [0.0_f64, 8.0_f64, 25.0_f64],
            pace_weight: 1.0_f64,
            efficiency_weight: 2.2_f64,
            critical_in_hand_weight: 3.0_f64,
            lost_score_ceiling_weight: 8.0_f64,
            empathy_weight: 0.0_f64,
            good_touch_penalty: 4.0_f64,
            clue_token_weight: 0.6_f64,
            clue_precision_weight: 0.0_f64,
            known_playable_weight: 0.0_f64,
            team_empathy_weight: 0.0_f64,
            resolved_cards_weight: 0.0_f64,
        }
    }
}

impl DefaultEvaluator {
    /// Weighted count of critical cards (last remaining copy of a still-needed card) in all hands.
    ///
    /// Each card contributes `overlap_bits / total_possibilities` where `overlap_bits` is the
    /// number of critical identities in its empathy set. This captures partially-identified
    /// critical cards (e.g. a card narrowed to [R5, B5] when both are critical contributes 1.0)
    /// rather than only fully-resolved ones.
    fn critical_cards_in_hand(table_state: &TableState, static_data: &StaticGameData) -> f64 {
        let variant = &static_data.variant;
        let num_players = static_data.number_of_players as usize;
        let stacks_size = variant.stacks_size as usize;

        // Build a bitmask of all critical card IDs.
        // A card is critical if exactly one copy remains outside the discard pile and it is still needed.
        let mut critical_mask: VariantCardsBitField = 0;
        for card_id in 0..variant.number_of_suits as usize * stacks_size {
            let total = variant.card_copies_count_by_id[card_id];
            if total == 0 {
                continue;
            }
            let discarded = table_state.discard_pile.copies_of(card_id as VariantCardId);
            let remaining = total.saturating_sub(discarded);
            if remaining != 1 {
                continue;
            }
            let suit = card_id / stacks_size;
            let rank_idx = card_id % stacks_size;
            if table_state.playing_stacks.stack_size(suit) as usize > rank_idx {
                continue;
            }
            critical_mask |= 1 << card_id;
        }

        if critical_mask == 0 {
            return 0.0;
        }

        let mut total = 0.0f64;
        for hand in table_state.hands[..num_players].iter() {
            for &deck_idx in hand.cards() {
                let empathy = table_state.deck.get_global_empathy(deck_idx);
                let overlap = empathy.as_bits() & critical_mask;
                if overlap != 0 {
                    let possibilities = empathy.count_possibilities();
                    if possibilities > 0 {
                        total += overlap.count_ones() as f64 / possibilities as f64;
                    }
                }
            }
        }
        total
    }

    /// Maximum score still achievable given the current discard pile.
    /// For each suit, walks ranks from the current stack top; stops at the first rank
    /// where all copies have been discarded (that rank and everything above it is lost).
    fn max_achievable_score(table_state: &TableState, static_data: &StaticGameData) -> u32 {
        let variant = &static_data.variant;
        let stacks_size = variant.stacks_size as usize;
        let mut total = 0u32;
        for suit in 0..variant.number_of_suits as usize {
            let already_played = table_state.playing_stacks.stack_size(suit) as usize;
            let mut suit_max = already_played;
            for rank_idx in already_played..stacks_size {
                let card_id = suit * stacks_size + rank_idx;
                let copies = variant.card_copies_count_by_id[card_id];
                if table_state.discard_pile.copies_of(card_id as VariantCardId) >= copies {
                    break;
                }
                suit_max += 1;
            }
            total += suit_max as u32;
        }
        total
    }

    /// Sum of identity-bits eliminated across all clued cards in every player's own hand.
    ///
    /// For each clued card, a player who knows exactly one identity (popcount=1) contributes
    /// `max_identities - 1` to the sum; a card with all identities open contributes 0.
    /// Higher is better — it means clues have conveyed more precise information.
    fn empathy_precision(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let max_identities =
            (static_data.variant.number_of_suits as u32) * (static_data.variant.stacks_size as u32);
        let mut total = 0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let clued_own = pk.own_hand & table_state.clue_touched_cards;
            let mut bits = clued_own;
            while bits != 0 {
                let lsb = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let card_deck_index = lsb as CardDeckIndex;
                let combined = pk.combined_possible_identities(card_deck_index, table_state, &static_data.variant);
                let popcount = combined.count_possibilities();
                total += (max_identities - popcount.min(max_identities)) as f64;
            }
        }
        total
    }

    /// Count of own-hand cards each player knows are playable, consulting knowledge in priority order:
    ///
    /// 1. `Signal::Play` — explicit convention instruction (covers finesses and blind-play setups;
    ///    the signal's `knowledge_updates` resolve any temporary identity desync when it unfolds).
    /// 2. `inferred_identities` — convention-inferred identity narrower than raw empathy
    ///    (field exists but currently unpopulated; check is future-proof).
    /// 3. Raw `empathy` — all possible identities fall within the current playable-cards mask.
    fn known_playable_in_hands(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let playable_mask = table_state.playable_cards(static_data);
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                // Priority 1: convention signal
                if pk.signals[idx].iter().any(|s| matches!(s, Signal::Play { .. })) {
                    total += 1.0;
                    continue;
                }
                // Priority 2: convention-inferred identity (currently unpopulated)
                if let Some(inferred) = pk.inferred_identities[idx] {
                    let bits = inferred.as_bits();
                    if bits != 0 && (bits & playable_mask) == bits {
                        total += 1.0;
                        continue;
                    }
                }
                // Priority 3: game-rule empathy from Deck (global)
                let bits = table_state.deck.get_global_empathy(idx as CardDeckIndex).as_bits();
                if bits != 0 && (bits & playable_mask) == bits {
                    total += 1.0;
                }
            }
        }
        total
    }

    /// Fraction of identity uncertainty eliminated across all own-hand cards for all players.
    ///
    /// Each card contributes `(max_identities − popcount) / max_identities` ∈ [0, 1).
    /// A fully-unknown card contributes 0; a fully-resolved card contributes near 1.
    /// Uses game-rule empathy from Deck — convention signals and inferred identities are captured
    /// separately by `known_playable_in_hands` and `resolved_card_count`.
    fn team_empathy_score(static_data: &StaticGameData, team_knowledge: &TeamKnowledge, table_state: &TableState) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let max_identities =
            (static_data.variant.number_of_suits as u32) * (static_data.variant.stacks_size as u32);
        let max_f = max_identities as f64;
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                let card_deck_index = idx as CardDeckIndex;
                let combined = pk.combined_possible_identities(card_deck_index, table_state, &static_data.variant);
                let popcount = combined.count_possibilities();
                total += (max_identities.saturating_sub(popcount.min(max_identities))) as f64 / max_f;
            }
        }
        total
    }

    /// Count of own-hand cards fully resolved to a single identity (`popcount == 1`).
    ///
    /// A sharper reward than `team_empathy_score`: fires only when a player knows exactly
    /// what a card is, enabling confident play or discard decisions.
    fn resolved_card_count(static_data: &StaticGameData, team_knowledge: &TeamKnowledge, table_state: &TableState) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                let card_deck_index = idx as CardDeckIndex;
                let combined = pk.combined_possible_identities(card_deck_index, table_state, &static_data.variant);
                if combined.is_exactly_known() {
                    total += 1.0;
                }
            }
        }
        total
    }
}

impl Evaluator for DefaultEvaluator {
    fn score(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> Score {
        self.score_breakdown(table_state, static_data, team_knowledge).total
    }

    fn score_breakdown(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> ScoreBreakdown {
        let game_score = self.score_weight * table_state.score(&static_data.variant) as f64;
        let strikes = table_state.strike_tokens as usize;
        let strike_penalty = self.strike_penalties.get(strikes).copied().unwrap_or(0.0);
        let pace = self.pace_weight * (table_state.pace(static_data)).max(-10) as f64;
        let efficiency_penalty =
            self.efficiency_weight * f64::from(table_state.required_efficiency(static_data));
        let critical_in_hand =
            self.critical_in_hand_weight * Self::critical_cards_in_hand(table_state, static_data);
        let theoretical_max =
            (static_data.variant.number_of_suits * static_data.variant.stacks_size) as f64;
        let lost_ceiling_penalty = self.lost_score_ceiling_weight
            * (theoretical_max - Self::max_achievable_score(table_state, static_data) as f64);
        let empathy_bonus = if self.empathy_weight != 0.0 {
            self.empathy_weight * Self::empathy_precision(table_state, static_data, team_knowledge)
        } else {
            0.0
        };
        let clue_tokens =
            self.clue_token_weight * table_state.clue_token_bank.whole_clue_tokens_count() as f64;
        let known_playable = if self.known_playable_weight != 0.0 {
            self.known_playable_weight
                * Self::known_playable_in_hands(table_state, static_data, team_knowledge)
        } else {
            0.0
        };
        let team_empathy = if self.team_empathy_weight != 0.0 {
            self.team_empathy_weight * Self::team_empathy_score(static_data, team_knowledge, table_state)
        } else {
            0.0
        };
        let resolved_cards = if self.resolved_cards_weight != 0.0 {
            self.resolved_cards_weight * Self::resolved_card_count(static_data, team_knowledge, table_state)
        } else {
            0.0
        };
        let total = game_score - strike_penalty + pace - efficiency_penalty + critical_in_hand
            - lost_ceiling_penalty
            + empathy_bonus
            + clue_tokens
            + known_playable
            + team_empathy
            + resolved_cards;
        ScoreBreakdown {
            game_score,
            strike_penalty,
            pace,
            efficiency_penalty,
            critical_in_hand,
            lost_ceiling_penalty,
            empathy_bonus,
            clue_tokens,
            known_playable,
            team_empathy,
            resolved_cards,
            total,
        }
    }

    fn clue_precision_bonus(
        &self,
        touched: &[u8],
        receiver: usize,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> Score {
        let precision_bonus = if self.clue_precision_weight != 0.0 {
            touched
                .iter()
                .filter(|&&idx| {
                    let card_deck_index = idx as CardDeckIndex;
                    let combined = team_knowledge.player(receiver).combined_possible_identities(card_deck_index, table_state, &static_data.variant);
                    combined.is_exactly_known()
                })
                .count() as f64
                * self.clue_precision_weight
        } else {
            0.0
        };

        let bad_touch_count = count_bad_touches(touched, receiver, table_state, static_data);

        precision_bonus - bad_touch_count as f64 * self.good_touch_penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
    use crate::game::clue_token_bank::ClueTokenBank;
    use crate::game::deck::Deck;
    use crate::game::hand::Hand;
    use crate::game::playing_stacks::PlayingStacks;
    use crate::game::state::table_state::TableState;
    use crate::game::static_game_data::StaticGameData;
    use crate::game::variant::test_variants::NO_VARIANT;

    fn make_state(strikes: u8) -> (TableState, StaticGameData) {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            Deck::new(&NO_VARIANT),
            Hand::empty_array(),
            0,
            0,
            PlayingStacks::empty(),
            strikes,
            CopiesCountingCardCollection::empty(),
        );
        (state, static_data)
    }

    #[test]
    fn higher_strikes_produce_lower_score() {
        let evaluator = DefaultEvaluator::default();
        let (s0, sd) = make_state(0);
        let (s1, _) = make_state(1);
        let (s2, _) = make_state(2);
        let tk = TeamKnowledge::new(3);
        assert!(evaluator.score(&s0, &sd, &tk) > evaluator.score(&s1, &sd, &tk));
        assert!(evaluator.score(&s1, &sd, &tk) > evaluator.score(&s2, &sd, &tk));
    }
}
