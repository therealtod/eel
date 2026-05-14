use crate::engine::convention::hgroup::h_group_core::count_bad_touches;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::decision_tree::Score;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::state::PlayerIndex;
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
    /// `misinformation_weight * misinformed_card_count` — penalty for own-hand cards whose effective
    /// inferred mask excludes the card's true identity (convention breakdown / misinformation).
    pub misinformation_penalty: f64,
    /// Sum of all terms above.
    pub total: f64,
}

impl std::fmt::Display for ScoreBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total={:.2} [score={:.1} -strike={:.1} +pace={:.1} -eff={:.1} +crit={:.1} -ceil={:.1} +emp={:.1} +clue={:.1} +play={:.1} +team_emp={:.1} -misinfo={:.1}]",
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
            self.misinformation_penalty,
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
            misinformation_penalty: 0.0,
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

    /// Penalty assessed when `actor` takes an action that ignores an active
    /// `Signal::Play` on one of their own untouched cards.
    ///
    /// Models the H-Group rule that a finessed (or otherwise blind-play-signalled)
    /// player must resolve the signal on their very next turn — delaying breaks
    /// the convention and corrupts downstream interpretations. State is the
    /// pre-action team knowledge / table state.
    fn signal_ignored_penalty(
        &self,
        _action: &GameAction,
        _actor: PlayerIndex,
        _static_data: &StaticGameData,
        _team_knowledge: &TeamKnowledge,
        _table_state: &TableState,
    ) -> Score {
        0.0
    }

    /// Immediate bonus for a play action that successfully advances a stack.
    ///
    /// Models the value of forward progress within the search horizon, separate from the leaf
    /// `game_score` term (which is symmetric for lines that reach the same total). Misplays
    /// (strikes) get 0 here; the strike penalty in the leaf already handles them.
    fn play_progress_bonus(
        &self,
        _action: &GameAction,
        _pre_action_state: &TableState,
        _post_action_state: &TableState,
        _static_data: &StaticGameData,
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
/// Per-clue immediate adjustments (applied to every clue action along the search line):
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
    /// `empathy_weight`, which applies to the leaf evaluation; this one fires once
    /// per clue action along the search line.
    pub clue_precision_weight: f64,
    /// Reward per card in any player's own hand where the entire empathy set is a subset
    /// of the currently playable cards (or the card carries a `Signal::Play`). Captures
    /// the value of clues that set up plays the search depth may not reach.
    pub known_playable_weight: f64,
    /// Reward proportional to the total fraction of identity uncertainty eliminated across
    /// all own-hand cards for all players: `Σ (max_ids − popcount) / max_ids`.
    /// Encourages states where the team knows more about every card, not just clued ones.
    pub team_empathy_weight: f64,
    /// Penalty applied to a non-Play action (or a Play of the wrong card) taken by an actor who
    /// holds an active `Signal::Play` on at least one untouched own-hand card. Captures the
    /// H-Group urgency rule: a finessed/blind-play-signalled card must be played on the very
    /// next turn, and stalling corrupts the convention.
    ///
    /// Set to 0 to disable.
    pub signal_ignored_penalty_weight: f64,
    /// Penalty per own-hand card whose effective inferred mask **excludes** the card's true
    /// identity (as seen by another player). Models the cost of a convention breakdown where
    /// a player is committed to a wrong reading — they will bomb or play incorrectly.
    ///
    /// Only fires when the truth is known to some other player (`visible_cards` bit set).
    /// Cards whose identity is unknown to all observers (freshly drawn in search) contribute 0.
    ///
    /// Set to 0 to disable. Suggested default: 3.0.
    pub misinformation_weight: f64,
    /// Reward per card successfully played to the stacks within the search horizon.
    ///
    /// Counteracts the structural bias where lines that play more cards lose ~1 pace point per
    /// extra draw at the leaf. Setting this to 1.0 exactly counteracts the drag, leaving
    /// the choice to the leaf's substantive signals (score, ceiling, misinformation).
    ///
    /// Set to 0 to disable.
    pub play_progress_weight: f64,
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
            signal_ignored_penalty_weight: 5.0_f64,
            misinformation_weight: 3.0_f64,
            play_progress_weight: 1.0_f64,
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

    /// Misinformation score per plan §4.3 — three-case formula summed over all own-hand cards.
    ///
    /// For each card whose truth is known (singleton in the omniscient deck):
    /// - `+0`               if `effective_mask` is a singleton equal to truth (exact knowledge).
    /// - `+w`               if `effective_mask` excludes truth entirely (committed to wrong id).
    /// - `+w * (n-1) / n`   if truth is present in the mask but `n > 1` (partial uncertainty).
    ///
    /// The formula unifies all three: when truth is in the mask, contribution = `w * (n-1)/n`,
    /// which is 0 for `n=1` and approaches `w` as `n` grows. When truth is excluded, `n=0`
    /// overlap → the hard `+w` branch fires instead.
    ///
    /// Truth is read from the omniscient deck (`table_state.deck`). Cards whose entry is not yet
    /// a singleton (freshly-drawn search cards) contribute 0.
    fn misinformation_score(
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let variant = &static_data.variant;
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                let card_deck_index = idx as CardDeckIndex;
                // Pull truth directly from the omniscient deck: singleton iff the card has been
                // revealed to spectators; multi-bit (freshly-drawn search card) means unknown.
                let truth = table_state.deck.get_global_empathy(card_deck_index);
                if !truth.is_exactly_known() {
                    continue;
                }
                let effective = pk.effective_inferred_mask(card_deck_index, variant);
                if effective.as_bits() & truth.as_bits() == 0 {
                    // Truth fully excluded: full penalty.
                    total += 1.0;
                } else {
                    // Truth present: partial penalty proportional to how many wrong identities
                    // the player also entertains.  (n-1)/n → 0 for exact knowledge, ~1 for wide
                    // uncertainty.
                    let n = effective.count_possibilities();
                    if n > 1 {
                        total += (n - 1) as f64 / n as f64;
                    }
                }
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
        let pace = self.pace_weight * (table_state.pace(static_data)).clamp(-10, static_data.number_of_players as i32) as f64;
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
        let misinformation_penalty = if self.misinformation_weight != 0.0 {
            self.misinformation_weight
                * Self::misinformation_score(static_data, team_knowledge, table_state)
        } else {
            0.0
        };
        let total = game_score - strike_penalty + pace - efficiency_penalty + critical_in_hand
            - lost_ceiling_penalty
            + empathy_bonus
            + clue_tokens
            + known_playable
            + team_empathy
            - misinformation_penalty;
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
            misinformation_penalty,
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

    fn signal_ignored_penalty(
        &self,
        action: &GameAction,
        actor: PlayerIndex,
        _static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> Score {
        if self.signal_ignored_penalty_weight == 0.0 {
            return 0.0;
        }
        let pk = team_knowledge.player(actor);
        let played_card: Option<CardDeckIndex> = match action {
            GameAction::Play { card_deck_index, .. } => Some(*card_deck_index),
            _ => None,
        };
        let mut any_signal = false;
        let mut hand_mask = pk.own_hand;
        while hand_mask != 0 {
            let idx = hand_mask.trailing_zeros() as CardDeckIndex;
            hand_mask &= hand_mask - 1;
            // Touched signalled cards are clued plays handled by PlayKnownPlayable; the
            // urgency rule applies only to untouched blind-play signals.
            if (table_state.clue_touched_cards >> idx) & 1 != 0 {
                continue;
            }
            if !pk.has_play_signal(idx) {
                continue;
            }
            any_signal = true;
            if played_card == Some(idx) {
                return 0.0; // signal honoured
            }
        }
        if any_signal {
            -self.signal_ignored_penalty_weight
        } else {
            0.0
        }
    }

    fn play_progress_bonus(
        &self,
        action: &GameAction,
        pre: &TableState,
        post: &TableState,
        static_data: &StaticGameData,
    ) -> Score {
        if self.play_progress_weight == 0.0 {
            return 0.0;
        }
        let GameAction::Play { .. } = action else { return 0.0; };
        if post.score(&static_data.variant) > pre.score(&static_data.variant) {
            self.play_progress_weight
        } else {
            0.0
        }
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

    #[test]
    fn signal_ignored_penalty_fires_when_actor_skips_signalled_card() {
        use crate::engine::convention::hgroup::signal::Signal;
        let evaluator = DefaultEvaluator::default();
        let (mut table_state, static_data) = make_state(0);
        // Put a single card (deck index 5) in player 0's hand, untouched, with a Signal::Play.
        table_state.hands[0] = Hand::new(&[5]);
        let mut tk = TeamKnowledge::new(3);
        tk.player_mut(0).own_hand = 1 << 5;
        tk.player_mut(0).signals[5].push(Signal::Play {
            card_deck_index: 5,
            committed_identity: 0,
            deadline_turn: 10,
        });

        // Discarding while signalled → full penalty.
        let discard = GameAction::Discard {
            player_index: 0,
            card_deck_index: 5,
            turn: 0,
        };
        let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, -evaluator.signal_ignored_penalty_weight);

        // A clue (different card / no play of the signalled card) also triggers the penalty.
        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![],
            clue: crate::game::clue::Clue {
                clue_type: crate::game::clue_type::ClueType::Rank,
                clue_value: 1,
            },
            turn: 0,
        };
        let pen = evaluator.signal_ignored_penalty(&clue, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, -evaluator.signal_ignored_penalty_weight);

        // Playing the signalled card → no penalty.
        let play = GameAction::Play {
            player_index: 0,
            card_deck_index: 5,
            turn: 0,
        };
        let pen = evaluator.signal_ignored_penalty(&play, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, 0.0);
    }

    #[test]
    fn signal_ignored_penalty_zero_when_no_active_signal() {
        let evaluator = DefaultEvaluator::default();
        let (mut table_state, static_data) = make_state(0);
        table_state.hands[0] = Hand::new(&[5]);
        let mut tk = TeamKnowledge::new(3);
        tk.player_mut(0).own_hand = 1 << 5;

        let discard = GameAction::Discard {
            player_index: 0,
            card_deck_index: 5,
            turn: 0,
        };
        let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, 0.0);
    }

    #[test]
    fn signal_ignored_penalty_ignores_touched_signalled_cards() {
        use crate::engine::convention::hgroup::signal::Signal;
        // A touched card with a Signal::Play is a clued play (PlayKnownPlayable territory),
        // not the H-Group "must blind-play next turn" rule. The urgency penalty must skip it.
        let evaluator = DefaultEvaluator::default();
        let (mut table_state, static_data) = make_state(0);
        table_state.hands[0] = Hand::new(&[5]);
        table_state.clue_touched_cards = 1 << 5;
        let mut tk = TeamKnowledge::new(3);
        tk.player_mut(0).own_hand = 1 << 5;
        tk.player_mut(0).signals[5].push(Signal::Play {
            card_deck_index: 5,
            committed_identity: 0,
            deadline_turn: 10,
        });

        let discard = GameAction::Discard {
            player_index: 0,
            card_deck_index: 5,
            turn: 0,
        };
        let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, 0.0);
    }

    /// §6.3 — misinformation term fires when effective mask excludes the true identity.
    ///
    /// Setup: 3-player game.  Card at deck-index 5 is in player-0's hand.  The omniscient
    /// deck says it is card-id 7 (singleton via `reveal_card`).  Player-0's knowledge says
    /// the card must be card-id 3 (disjoint from the truth).
    ///
    /// Expected: `misinformation_score` > 0.  When the knowledge is corrected to include
    /// the truth, the score drops.
    #[test]
    fn misinformation_term_fires_on_excluded_truth() {
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;

        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };

        // Build a table state whose deck reveals card-id 7 at deck-index 5.
        let mut deck = Deck::new(&NO_VARIANT);
        deck.reveal_card(5, 7); // truth = card 7 at position 5

        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            Hand::empty_array(),
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );

        // Player-0's knowledge: card 5 must be card-id 3 (mask = 1<<3), which excludes truth (1<<7).
        let mut tk = TeamKnowledge::new(3);
        let mut pk0 = knowledge_for_hand(&[5]);
        pk0.inferred_identities[5] = Some(crate::game::card::CardIdentityMask::from_bits(1 << 3));
        *tk.player_mut(0) = pk0;

        let score_misinformed =
            DefaultEvaluator::misinformation_score(&static_data, &tk, &state);
        assert!(
            score_misinformed > 0.0,
            "misinformation_score should be positive when effective mask excludes truth (got {score_misinformed})"
        );

        // Correct the knowledge: effective mask now includes the truth.
        tk.player_mut(0).inferred_identities[5] =
            Some(crate::game::card::CardIdentityMask::from_bits(1 << 7));
        let score_correct = DefaultEvaluator::misinformation_score(&static_data, &tk, &state);
        assert_eq!(
            score_correct, 0.0,
            "misinformation_score should be 0 when knowledge exactly matches truth"
        );
    }
}
