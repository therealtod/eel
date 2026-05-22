use crate::engine::convention::hgroup::h_group_core::{
    count_bad_touches, get_chop_index, is_potential_bad_touch,
};
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::decision_tree::Score;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
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
    /// `critical_exposure_weight * critical_exposure_score` — penalty for truth-critical cards
    /// in hands, scaled by how close each one is to being discarded.
    pub critical_exposure_penalty: f64,
    /// `lost_score_ceiling_weight * (theoretical_max − max_achievable)` — penalty for lost score ceiling.
    pub lost_ceiling_penalty: f64,
    /// `empathy_weight * empathy_precision` — reward for narrower identity ranges (disabled by default).
    pub empathy_bonus: f64,
    /// `clue_token_weight * harmonic(count) * (1 + clue_demand_weight * demand)` — scarcity-weighted reward for clue tokens.
    pub clue_tokens: f64,
    /// `known_playable_weight * known_playable_in_hands` — reward for cards known (by their owner) to be playable.
    pub known_playable: f64,
    /// `team_empathy_weight * Δ team_empathy_score` accumulated across clue actions on the
    /// search line. Rewards clues that tighten team-wide identity uncertainty. Always 0.0 at
    /// a pure leaf evaluation — the value is folded in by `team_empathy_delta_bonus` as the
    /// search walks each ply.
    pub team_empathy: f64,
    /// `misinformation_weight * misinformed_card_count` — penalty for own-hand cards whose effective
    /// inferred mask excludes the card's true identity (convention breakdown / misinformation).
    pub misinformation_penalty: f64,
    /// `critical_exposure_delta_weight * Δ critical_chop_count` accumulated as an immediate
    /// bonus/penalty per turn. Always 0.0 at a pure leaf evaluation — the value is folded in
    /// by `critical_exposure_delta_bonus` as the search walks each ply.
    pub critical_exposure_delta: f64,
    /// Sum of all terms above.
    pub total: f64,
}

impl std::fmt::Display for ScoreBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total={:.2} [score={:.1} -strike={:.1} +pace={:.1} -eff={:.1} -crit_exp={:.1} -ceil={:.1} +emp={:.1} +clue={:.1} +play={:.1} +team_emp={:.1} -misinfo={:.1} ±crit_delta={:.1}]",
            self.total,
            self.game_score,
            self.strike_penalty,
            self.pace,
            self.efficiency_penalty,
            self.critical_exposure_penalty,
            self.lost_ceiling_penalty,
            self.empathy_bonus,
            self.clue_tokens,
            self.known_playable,
            self.team_empathy,
            self.misinformation_penalty,
            self.critical_exposure_delta,
        )
    }
}

/// Trait for scoring a leaf game state during search.
///
/// Truth-aware methods take a `truth: &dyn PlayerPOV` reference, which is the root
/// searcher's POV held fixed across the rollout. It resolves the actual identity of
/// cards visible to the searcher (other players' hands, played/discarded cards). This
/// is required to see through clue-induced public-empathy narrowing — e.g. detecting
/// that a touched card is trash even after a clue widens its public empathy via GTP.
pub trait Evaluator: Send + Sync {
    /// Score the leaf state. Receives a full [`KnowledgeAwareGameState`] so the evaluator
    /// can consult engine-only signals like phantom plays (successful plays whose stack
    /// assignment was deferred). `phantom_plays` is the accumulated count from the search path.
    fn score(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> Score;

    /// Per-term breakdown of the score. The default implementation returns only the total;
    /// override this to expose individual contributions for debugging.
    fn score_breakdown(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> ScoreBreakdown {
        ScoreBreakdown {
            game_score: 0.0,
            strike_penalty: 0.0,
            pace: 0.0,
            efficiency_penalty: 0.0,
            critical_exposure_penalty: 0.0,
            lost_ceiling_penalty: 0.0,
            empathy_bonus: 0.0,
            clue_tokens: 0.0,
            known_playable: 0.0,
            team_empathy: 0.0,
            misinformation_penalty: 0.0,
            critical_exposure_delta: 0.0,
            total: self.score(state, truth, phantom_plays),
        }
    }

    /// Immediate bonus/penalty for a clue action.
    ///
    /// `touched` are the deck indices directly touched by the clue.
    /// `receiver` is the clue target; `giver` is the acting player.
    fn clue_precision_bonus(
        &self,
        _touched: &[u8],
        _receiver: usize,
        _giver: usize,
        _truth: &dyn PlayerPOV,
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

    /// Immediate bonus for a play action that successfully advances the engine's effective
    /// score (real stack progress *or* a phantom play).
    ///
    /// Models the value of forward progress within the search horizon, separate from the leaf
    /// `game_score` term (which is symmetric for lines that reach the same total). Misplays
    /// (strikes) get 0 here; the strike penalty in the leaf already handles them.
    fn play_progress_bonus(
        &self,
        _action: &GameAction,
        _pre_action_state: &KnowledgeAwareGameState,
        _post_action_state: &KnowledgeAwareGameState,
        _pre_phantom_plays: u8,
        _post_phantom_plays: u8,
    ) -> Score {
        0.0
    }

    /// Penalty assessed when `actor` chooses to `Discard`.
    ///
    /// Combines two risks the chop-discard pricing should reflect:
    ///
    /// 1. **Bottom Deck Risk (BDR):** the discarded card has wide empathy; some of its
    ///    candidate identities have all-but-one copy already accounted for, so discarding
    ///    *this* copy would make that identity newly critical. If the remaining copy is not
    ///    visible in any player's hand, it might be at the bottom of the deck and lost —
    ///    a ceiling-loss risk weighted by the candidate's probability under the empathy.
    ///
    /// 2. **Discard while holding a known-playable:** the team expected this actor to play
    ///    a globally-known-playable; choosing to discard instead means the team did not
    ///    save a critical onto chop on the prior turn, so this discard may dump a critical.
    ///    Modeled as a flat penalty.
    ///
    /// Returns a non-positive `Score` (penalty). Default zero.
    fn discard_action_penalty(
        &self,
        _action: &GameAction,
        _actor: PlayerIndex,
        _pre_action_state: &KnowledgeAwareGameState,
        _truth: &dyn PlayerPOV,
    ) -> Score {
        0.0
    }

    /// Immediate bonus for a clue action that tightens team-wide empathy: the change in
    /// `team_empathy_score` from before to after the clue, weighted by `team_empathy_weight`.
    ///
    /// Fires only on `GameAction::Clue`. Modeled as a delta rather than a leaf term because
    /// the static leaf measurement penalises any line that draws a fresh card (the new card
    /// has maximum uncertainty), creating a draw-dilution bias that punishes plays. Clueing
    /// is the only action that meaningfully *adds* team identity information; rewarding the
    /// delta at clue time captures the value without taxing plays/discards.
    fn team_empathy_delta_bonus(
        &self,
        _action: &GameAction,
        _pre_action_state: &KnowledgeAwareGameState,
        _post_action_state: &KnowledgeAwareGameState,
    ) -> Score {
        0.0
    }

    /// Per-turn immediate bonus/penalty tied to changes in critical-card chop exposure.
    ///
    /// For each teammate whose chop holds a truth-critical card before `action`:
    /// - **Reward** `+critical_exposure_delta_weight` if `action` removed the exposure (the
    ///   chop card is now clue-touched in the post-action state).
    /// - **Penalty** `−critical_exposure_delta_weight` if the card is still exposed **and**
    ///   the actor had at least one clue token available (a save clue was possible but skipped).
    ///
    /// Truth identity from `truth: &dyn PlayerPOV` is required to identify critical chop cards
    /// outside the root player's own hand. Cards whose identity is unknown to the searcher
    /// (draw-fresh in search) contribute 0.
    fn critical_exposure_delta_bonus(
        &self,
        _action: &GameAction,
        _actor: PlayerIndex,
        _pre_action_state: &KnowledgeAwareGameState,
        _post_action_state: &KnowledgeAwareGameState,
        _truth: &dyn PlayerPOV,
    ) -> Score {
        0.0
    }

    /// Optimistic upper bound on the best score reachable from `table_state` with `depth`
    /// more plies of search remaining (including accumulated immediate bonuses).
    ///
    /// The default returns `f64::INFINITY` (no pruning). Implementations that return a
    /// finite bound enable Branch-and-Bound pruning in `best_score_at_depth`.
    fn upper_bound(
        &self,
        _table_state: &TableState,
        _static_data: &StaticGameData,
        _depth: usize,
    ) -> Score {
        f64::INFINITY
    }
}

const fn build_reciprocal_lut() -> [f64; 65] {
    let mut arr = [0.0f64; 65];
    let mut i = 1usize;
    while i <= 64 {
        arr[i] = 1.0 / i as f64;
        i += 1;
    }
    arr
}

/// Precomputed 1/n for n = 0..=64. Index 0 maps to 0.0 (sentinel for zero-possibility cards).
const RECIPROCAL_LUT: [f64; 65] = build_reciprocal_lut();

const fn build_harmonic_lut() -> [f64; 9] {
    let mut arr = [0.0f64; 9];
    let mut i = 1usize;
    while i <= 8 {
        arr[i] = arr[i - 1] + 1.0 / i as f64;
        i += 1;
    }
    arr
}

/// Precomputed harmonic numbers H(n) = 1 + 1/2 + … + 1/n for n = 0..=8.
const HARMONIC_LUT: [f64; 9] = build_harmonic_lut();

/// Default heuristic evaluator.
///
/// Scoring terms:
/// - `score_weight * game_score`                        — reward progress
/// - `-strike_penalty(strikes)`                         — steep penalty near 3 strikes
/// - `pace_weight * pace` (clamped)                     — reward breathing room
/// - `-efficiency_weight * required_efficiency`         — penalise remaining discard burden
/// - `-critical_exposure_weight * critical_exposure_score` — penalise truth-critical cards in hands, scaled by discard imminence
/// - `-lost_score_ceiling_weight * lost_score_ceiling`  — penalise any reduction in max achievable score
/// - `empathy_weight * empathy_precision`               — reward narrower inferred identity ranges on clued cards
/// - `clue_token_weight * harmonic(n) * (1 + clue_demand_weight * demand)` — scarcity-weighted token value
///
/// Per-clue immediate adjustments (applied to every clue action along the search line):
/// - `empathy_weight * resolved_touched_cards`          — precision bonus for clues that fully resolve touched cards
/// - `-good_touch_penalty * bad_touch_count`            — penalty for each touched card with no overlap with still-needed
///                                                        cards (good-touch principle violation)
/// - `-potential_bad_touch_penalty`                     — flat penalty when a touched-on-receiver still-needed
///                                                        identity overlaps with an identity the giver likely
///                                                        already holds (potential duplicate)
pub struct DefaultEvaluator {
    /// Multiplier for the Hanabi game score term; dominates the total and keeps score progress as the primary objective.
    pub score_weight: f64,
    /// Per-strike penalty values indexed by strike count (0, 1, 2, 3).
    /// Index 3 applies to the terminal 3-strike game-over state and must be large enough
    /// that any continuing game scores higher than a struck-out one.
    pub strike_penalties: [f64; 4],
    /// Multiplier for the pace term (`pace = clues_left + cards_left − cards_needed`), clamped to −10 from below.
    pub pace_weight: f64,
    /// Multiplier for the required-efficiency penalty; higher values penalise states that demand many future discards.
    pub efficiency_weight: f64,
    /// Multiplier for the per-card `critical_exposure_score`. The score for each truth-critical
    /// card held in a player's hand is `1 / (1 + (rank_from_chop - 1) + 4 * buffer_count)`, where:
    ///
    /// - `rank_from_chop` is 1 if the card is the player's current chop-eligible card, 2 if the
    ///   next chop-eligible slot to its left, etc. Only counts cards that are untouched and not
    ///   known-trash (i.e., the discard order from the holder's perspective).
    /// - `buffer_count` is the number of known-playables plus known-trash cards in the holder's
    ///   hand. Each one represents an action (play or trash-discard) the holder will take before
    ///   being forced to discard the critical card.
    ///
    /// Buffer dominates position (factor 4): one buffer action ≈ moving the card four slots
    /// away from chop, so even the rightmost-but-buffered critical is safer than the
    /// untouched-and-unbuffered slot-1 critical.
    ///
    /// Cards classified as **touched but not known-playable** (clued saves, prompts, …) are
    /// neither buffer nor chop-eligible — the holder will keep them indefinitely; they
    /// contribute zero to the score. Touched + known-playable counts as buffer.
    ///
    /// Truth identity comes from `truth: &dyn PlayerPOV`, so fresh draws (no truth identity)
    /// contribute zero. This is intentional: the term reflects *known* critical risk, not
    /// hypothetical risk from drawing into a fresh slot.
    pub critical_exposure_weight: f64,
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
    /// Base multiplier for the clue-token term. Value is `clue_token_weight * harmonic(n)`,
    /// where `harmonic(n) = Σ_{i=1}^{n} 1/i`. The 0→1 jump is the largest; each additional
    /// token adds diminishing value, reflecting that scarcity matters most at very low counts.
    /// If `clue_demand_weight > 0`, the result is further scaled by a demand factor.
    ///
    /// Note: `harmonic(n)` grows without bound (slowly), so the term keeps rising past 8 tokens.
    /// We don't model the negative marginal value of being *forced* to clue near the cap; in
    /// practice the team rarely sits at max for long, so the simplification is acceptable.
    ///
    /// Scale reference for tuning: with `clue_token_weight = 1.0`, the term contributes ~1.0 at
    /// 1 token, ~2.08 at 4 tokens, ~2.72 at 8 tokens (before any demand multiplier). Picking a
    /// value requires re-balancing against `pace_weight`, `efficiency_weight`, etc., since the
    /// substitution from linear `n` to `H(n)` is *not* a simple scalar rescale.
    pub clue_token_weight: f64,
    /// Scales the demand factor that amplifies clue-token value when there are many unclued
    /// critical or playable cards in hands. Demand is the sum over all unclued hand cards of
    /// `P(card is critical) + P(card is playable)`. The final token value is
    /// `clue_token_weight * harmonic(n) * (1 + clue_demand_weight * demand)`.
    /// Set to 0 to disable.
    pub clue_demand_weight: f64,
    /// Immediate reward per touched card that is fully resolved to a single identity
    /// after the clue is applied (good-touch precision bonus). Distinct from
    /// `empathy_weight`, which applies to the leaf evaluation; this one fires once
    /// per clue action along the search line.
    pub clue_precision_weight: f64,
    /// Reward per card in any player's own hand where the entire empathy set is a subset
    /// of the currently playable cards (or the card carries a `Signal::Play`). Captures
    /// the value of clues that set up plays the search depth may not reach.
    pub known_playable_weight: f64,
    /// Reward proportional to the per-clue *change* in total fraction of identity uncertainty
    /// eliminated across all own-hand cards for all players, `Σ (max_ids − popcount) / max_ids`.
    ///
    /// Applied as an immediate bonus on each clue action along the search line (see
    /// `team_empathy_delta_bonus`) — not as a static leaf term. Measuring the absolute value
    /// at the leaf created a draw-dilution bias: a freshly drawn card has near-maximum
    /// uncertainty and lowers the sum, so any line with an extra play (= extra draw) looked
    /// worse on this term even when it strictly improved the game state.
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
    /// Multiplier for the **Bottom Deck Risk (BDR)** score charged on `Discard` actions.
    ///
    /// For each candidate identity `id` in the discarded card's empathy, if discarding this
    /// copy would make `id` newly critical (`remaining_copies − 1 == 1`) **and** no other
    /// player's hand visibly holds the surviving copy (truth POV), the term contributes
    /// `P(card_is_id) × P(surviving_copy_lost) × loss_if_lost(id)` where:
    ///
    /// - `P(card_is_id)` = `1 / popcount` — uniform over empathy (Term A).
    /// - `P(surviving_copy_lost)` = `1 / max(1, deck_remaining)` — Laplace bottom-card
    ///   probability; small with a full deck, approaches 1 as the deck empties (Term B).
    /// - `loss_if_lost` = achievable ranks from `rank_idx` upward, stopping at the first
    ///   rank already fully in the discard pile (Term C).
    ///
    /// Terminal-rank ids are skipped (single-copy by construction).
    ///
    /// Set to 0 to disable.
    pub bottom_deck_risk_weight: f64,
    /// Flat penalty applied when the actor chooses `Discard` while at least one card in
    /// their own hand is globally known-playable (a clued play, a `Signal::Play`, or empathy
    /// fully inside the playable mask).
    ///
    /// Models the H-Group expectation that a player with a known-playable plays it; a
    /// discard instead implies the team did not save a critical onto chop, so the chop
    /// being dumped may itself be critical. The penalty is large enough to prefer the play.
    ///
    /// Set to 0 to disable.
    pub discard_while_known_playable_penalty: f64,
    /// Flat penalty applied when the clue may duplicate, in the receiver's hand, a still-needed
    /// identity the giver likely already holds.
    ///
    /// Fires when the OR of touched-on-receiver truth identities intersects the OR of inferred
    /// identities of the giver's own-hand cards that are themselves touched or play-signaled,
    /// after filtering trash from both sides.
    ///
    /// Set to 0 to disable.
    pub potential_bad_touch_penalty: f64,
    /// Immediate bonus/penalty per critical-card chop situation per turn.
    ///
    /// Applied as an immediate per-turn delta (not a leaf term). For each teammate whose chop
    /// holds a truth-critical card before the action:
    /// - `+critical_exposure_delta_weight` if the actor clue-touched the chop card (saved it).
    /// - `−critical_exposure_delta_weight` if the card remains unsaved **and** the actor had
    ///   at least one clue token (a save was available but was skipped).
    ///
    /// Set to 0 to disable.
    pub critical_exposure_delta_weight: f64,
}

impl Default for DefaultEvaluator {
    fn default() -> Self {
        DefaultEvaluator {
            score_weight: 10.0_f64,
            strike_penalties: [0.0_f64, 10.0_f64, 30.0_f64, 1000.0_f64],
            pace_weight: 1.0_f64,
            efficiency_weight: 1.9_f64,
            critical_exposure_weight: 3.0_f64,
            lost_score_ceiling_weight: 8.0_f64,
            empathy_weight: 0.0_f64,
            good_touch_penalty: 20.0_f64,
            clue_token_weight: 0.6_f64,
            clue_demand_weight: 0.05_f64,
            clue_precision_weight: 0.0_f64,
            known_playable_weight: 1.0_f64,
            team_empathy_weight: 0.3_f64,
            signal_ignored_penalty_weight: 5.0_f64,
            misinformation_weight: 0.0_f64,
            play_progress_weight: 1.0_f64,
            potential_bad_touch_penalty: 5.0_f64,
            bottom_deck_risk_weight: 2.0_f64,
            discard_while_known_playable_penalty: 8.0_f64,
            critical_exposure_delta_weight: 0.5_f64,
        }
    }
}

impl DefaultEvaluator {
    /// Bitmask of card identities that are critical: exactly one copy remains outside the
    /// discard pile and the card is still needed (not yet on the play stacks).
    fn critical_mask(
        table_state: &TableState,
        static_data: &StaticGameData,
    ) -> VariantCardsBitField {
        let variant = &static_data.variant;
        let stacks_size = variant.stacks_size as usize;

        let mut mask: VariantCardsBitField = 0;
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
            mask |= 1 << card_id;
        }
        mask
    }

    /// Discard-imminence threat sum over every truth-critical card across all hands.
    ///
    /// See [`Self::critical_exposure_weight`] for the formula and rationale. Returns 0 when no
    /// card identity is currently critical, or when no critical cards live in any hand.
    fn critical_exposure_score(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        truth: &dyn PlayerPOV,
    ) -> f64 {
        let critical_mask = Self::critical_mask(table_state, static_data);
        if critical_mask == 0 {
            return 0.0;
        }
        let num_players = static_data.number_of_players as usize;
        let playable_mask = table_state.playable_cards(static_data);
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let pov = LightweightPlayerPOV::new(p, pk, team_knowledge, table_state, static_data);
            let hand = &table_state.hands[p];

            // First pass: classify each slot from p's POV and build the chop-order list
            // (right-to-left through the hand) of untouched-non-known-trash cards.
            let mut buffer: u32 = 0;
            let mut chop_eligible: smallvec::SmallVec<[CardDeckIndex; 6]> =
                smallvec::SmallVec::new();
            for &deck_idx in hand.cards().iter().rev() {
                let known_trash = pov.is_known_trash(deck_idx);
                let known_playable = Self::card_known_playable(&pov, deck_idx, playable_mask);
                if known_trash || known_playable {
                    buffer += 1;
                    continue;
                }
                if pov.is_touched(deck_idx) {
                    continue; // touched non-playable: held indefinitely, not at risk
                }
                chop_eligible.push(deck_idx);
            }

            // Second pass: each truth-critical card in the chop-eligible list contributes
            // a threat sized by buffer + rank-from-chop.
            for (rank_from_chop, &deck_idx) in chop_eligible.iter().enumerate() {
                // Unknown identity → no contribution here; probabilistic risk is BDR's job.
                let Some(id) = truth.card_identity(deck_idx) else {
                    continue;
                };
                if (1u64 << id) & critical_mask == 0 {
                    continue;
                }
                let r = rank_from_chop as f64; // 0 = chop
                let threat = 1.0 / (1.0 + r + 4.0 * buffer as f64);
                total += threat;
            }
        }
        total
    }

    /// True iff the holder of `deck_idx` (the POV's player) would have a play
    /// action available for that card.
    ///
    /// Mirrors the candidate predicates of the search's actual play generators
    /// so penalties and exposure buffers cannot fire for cards the search could
    /// not have played:
    /// - `Signal::Play` → `BlindPlay` proposes it.
    /// - `pov.inferred_identities(idx) ⊆ playable_mask` → `PlayKnownPlayable`
    ///   proposes it.
    ///
    /// The raw deck-empathy fallback that used to live here was unsound for the
    /// KP penalty — it could fire on cards neither tech would offer as a play,
    /// trapping the actor between a strike (if she plays) and the penalty (if
    /// she doesn't).
    fn card_known_playable(
        pov: &dyn PlayerPOV,
        deck_idx: CardDeckIndex,
        playable_mask: VariantCardsBitField,
    ) -> bool {
        let pk = pov.team_knowledge().player(pov.player_index());
        if pk.signals[deck_idx as usize]
            .iter()
            .any(|s| matches!(s, Signal::Play { .. }))
        {
            return true;
        }
        let bits = pov.inferred_identities(deck_idx).as_bits();
        bits != 0 && (bits & playable_mask) == bits
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
                let combined = pk.combined_possible_identities(
                    card_deck_index,
                    table_state,
                    &static_data.variant,
                );
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
    ///
    /// A card is **not** credited if the searcher's truth view reveals it to be unplayable,
    /// even when the receiver's public knowledge appears playable. This guards against
    /// rewarding clues that mislead a player into believing a trash card is a playable —
    /// the receiver's optimistic reading is a misinformation cost, not an asset.
    fn known_playable_in_hands(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        truth: &dyn PlayerPOV,
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
                // Detect the misinformation case: searcher knows the truth and it's
                // unplayable. Skip even if the holder's local knowledge thinks otherwise.
                if let Some(truth_id) = truth.card_identity(idx as CardDeckIndex) {
                    if (1u64 << truth_id) & playable_mask == 0 {
                        continue;
                    }
                }
                // Priority 1: convention signal
                if pk.signals[idx]
                    .iter()
                    .any(|s| matches!(s, Signal::Play { .. }))
                {
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
                let bits = table_state
                    .deck
                    .get_global_empathy(idx as CardDeckIndex)
                    .as_bits();
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
    fn team_empathy_score(
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let max_identities =
            (static_data.variant.number_of_suits as u32) * (static_data.variant.stacks_size as u32);
        let inv_max = 1.0 / max_identities as f64;
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                let card_deck_index = idx as CardDeckIndex;
                let combined = pk.combined_possible_identities(
                    card_deck_index,
                    table_state,
                    &static_data.variant,
                );
                let popcount = combined.count_possibilities();
                total +=
                    (max_identities.saturating_sub(popcount.min(max_identities))) as f64 * inv_max;
            }
        }
        total
    }

    /// Harmonic sum H(n) = 1 + 1/2 + 1/3 + … + 1/n.
    ///
    /// Models the scarcity value of a clue bank with `n` whole tokens: the first token is worth
    /// 1.0, the second 0.5, the third 0.33, etc. Going from 0 → 1 is the largest jump; going
    /// from 7 → 8 adds only 0.125.
    fn harmonic(n: u8) -> f64 {
        HARMONIC_LUT[n as usize]
    }

    /// Fractional count of unclued cards in hands that "need" a clue — either potentially
    /// playable or potentially critical.
    ///
    /// For each card not yet clue-touched, the contribution is
    /// `max(P(playable), P(critical))` based on its empathy set. We take the max rather than
    /// the sum so cards that are *both* (e.g. a 5 atop a 4-stack) don't double-count.
    ///
    /// Note: this only excludes already-touched cards. A touched card that still needs further
    /// disambiguation (e.g. clued by color but not rank) is treated as zero demand — an
    /// approximation. The signal is meant to capture broad pressure, not exact token need.
    fn clue_demand(table_state: &TableState, static_data: &StaticGameData) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let playable_mask = table_state.playable_cards(static_data);
        let critical_mask = Self::critical_mask(table_state, static_data);

        let mut demand = 0.0f64;
        for hand in table_state.hands[..num_players].iter() {
            for &deck_idx in hand.cards() {
                if (table_state.clue_touched_cards >> deck_idx) & 1 != 0 {
                    continue;
                }
                let empathy = table_state.deck.get_global_empathy(deck_idx);
                let bits = empathy.as_bits();
                let possibilities = empathy.count_possibilities() as usize;
                let playable_overlap = (bits & playable_mask).count_ones();
                let critical_overlap = (bits & critical_mask).count_ones();
                demand +=
                    playable_overlap.max(critical_overlap) as f64 * RECIPROCAL_LUT[possibilities];
            }
        }
        demand
    }

    /// Misinformation score per plan §4.3 — three-case formula summed over all own-hand cards.
    ///
    /// For each card whose truth is known (visible to the searcher, OR — for own cards —
    /// narrowed to a singleton in their effective view):
    /// - `+0`               if `effective_mask` is a singleton equal to truth (exact knowledge).
    /// - `+w`               if `effective_mask` excludes truth entirely (committed to wrong id).
    /// - `+w * (n-1) / n`   if truth is present in the mask but `n > 1` (partial uncertainty).
    ///
    /// The formula unifies all three: when truth is in the mask, contribution = `w * (n-1)/n`,
    /// which is 0 for `n=1` and approaches `w` as `n` grows. When truth is excluded, `n=0`
    /// overlap → the hard `+w` branch fires instead.
    ///
    /// Truth comes from the searcher's POV — which sees other players' hands and any
    /// publicly revealed cards. Cards the searcher cannot resolve (freshly-drawn search
    /// cards) contribute 0.
    fn misinformation_score(
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        _table_state: &TableState,
        truth: &dyn PlayerPOV,
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
                let Some(truth_id) = truth.card_identity(card_deck_index) else {
                    continue;
                };
                let truth_bits = 1u64 << truth_id;
                let effective = pk.effective_inferred_mask(card_deck_index, variant);
                if effective.as_bits() & truth_bits == 0 {
                    // Truth fully excluded: full penalty.
                    total += 1.0;
                } else {
                    // Truth present: partial penalty proportional to how many wrong identities
                    // the player also entertains.  (n-1)/n → 0 for exact knowledge, ~1 for wide
                    // uncertainty.
                    let n = effective.count_possibilities() as usize;
                    if n > 1 {
                        total += 1.0 - RECIPROCAL_LUT[n];
                    }
                }
            }
        }
        total
    }

    /// Bottom Deck Risk score for discarding `discarded_deck_idx` from `table_state`.
    ///
    /// Uses a three-term expected-value model per eligible identity `id`:
    ///
    ///   `P(card_is_id)  ×  P(surviving_copy_lost)  ×  loss_if_lost(id)`
    ///
    /// - **Term A** `P(card_is_id)` = `1 / popcount` — uniform over empathy.
    /// - **Term B** `P(surviving_copy_lost)` = `1 / max(1, deck_remaining)` — Laplace
    ///   bottom-card probability; low with a full deck, rising toward 1 as it empties.
    ///   This is the dominant correction over the old worst-case model. See
    ///   `bottom_deck_risk_weight` doc for the probabilistic counterpart to
    ///   `critical_exposure_weight` (which handles known-critical risk).
    /// - **Term C** `loss_if_lost` = achievable ranks from `rank_idx` upward before the
    ///   first fully-discarded rank (already-lost ranks are excluded from the loss).
    ///
    /// An identity is skipped when: it has only one total copy (terminal); its rank is
    /// the top of the stack; its suit is already past this rank; discarding doesn't drive
    /// remaining copies to exactly 1; or any other hand visibly holds the surviving copy.
    fn bottom_deck_risk_score(
        discarded_deck_idx: CardDeckIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
    ) -> f64 {
        let variant = &static_data.variant;
        let stacks_size = variant.stacks_size as usize;
        // When the truth player can see the discarded card (it's in another player's hand),
        // use the resolved singleton identity instead of the public empathy distribution.
        // This prevents spurious BDR charges for cards the truth player knows are safe
        // (e.g. a known trash card, or a card whose surviving copy is visibly held).
        let (empathy, popcount) = if let Some(known_id) = truth.card_identity(discarded_deck_idx) {
            (1u64 << known_id, 1.0f64)
        } else {
            let e = table_state
                .deck
                .get_global_empathy(discarded_deck_idx)
                .as_bits();
            if e == 0 {
                return 0.0;
            }
            (e, e.count_ones() as f64)
        };
        let num_players = static_data.number_of_players as usize;
        let mut visible_ids: VariantCardsBitField = 0;
        for hand in table_state.hands[..num_players].iter() {
            for &di in hand.cards() {
                if di == discarded_deck_idx {
                    continue;
                }
                if let Some(id) = truth.card_identity(di) {
                    visible_ids |= 1u64 << id;
                }
            }
        }
        // Term B: Laplace probability the surviving copy is at the bottom of the deck.
        let deck_remaining = table_state.deck.current_size as f64;
        let p_surviving_lost = 1.0 / deck_remaining.max(1.0);
        let mut score = 0.0f64;
        let mut bits = empathy;
        while bits != 0 {
            let id = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            let total_copies = variant.card_copies_count_by_id[id] as usize;
            if total_copies <= 1 {
                continue;
            }
            let rank_idx = id % stacks_size;
            if rank_idx + 1 == stacks_size {
                continue; // terminal rank — single-copy by construction
            }
            let suit = id / stacks_size;
            if table_state.playing_stacks.stack_size(suit) as usize > rank_idx {
                continue; // already on the stack: card is trash, no BDR
            }
            let already_discarded =
                table_state.discard_pile.copies_of(id as VariantCardId) as usize;
            if total_copies.saturating_sub(already_discarded + 1) != 1 {
                continue;
            }
            if (visible_ids >> id) & 1 != 0 {
                continue;
            }
            // Term A: uniform probability over empathy candidates.
            let p_card_is_id = 1.0 / popcount;
            // Term C: achievable ranks lost — walk up from rank_idx, stop at first fully-
            // discarded rank (those were already unreachable before this discard).
            let mut loss_count = 0usize;
            for r in rank_idx..stacks_size {
                let cid = suit * stacks_size + r;
                let total_r = variant.card_copies_count_by_id[cid] as usize;
                if table_state.discard_pile.copies_of(cid as VariantCardId) as usize >= total_r {
                    break;
                }
                loss_count += 1;
            }
            score += p_card_is_id * p_surviving_lost * loss_count as f64;
        }
        score
    }

    /// True iff `actor` currently holds at least one card the search's play
    /// generators would offer them — i.e. a card matched by either
    /// `BlindPlay` (via `Signal::Play`) or `PlayKnownPlayable` (via combined
    /// inferred ⊆ playable). Delegates to [`Self::card_known_playable`].
    fn actor_has_known_playable(
        actor: PlayerIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
    ) -> bool {
        let pk = team_knowledge.player(actor);
        let pov = LightweightPlayerPOV::new(actor, pk, team_knowledge, table_state, static_data);
        let playable_mask = table_state.playable_cards(static_data);
        let mut hand = pk.own_hand;
        while hand != 0 {
            let idx = hand.trailing_zeros() as CardDeckIndex;
            hand &= hand - 1;
            if Self::card_known_playable(&pov, idx, playable_mask) {
                return true;
            }
        }
        false
    }

    /// Count of own-hand cards fully resolved to a single identity (`popcount == 1`).
    ///
    /// A sharper reward than `team_empathy_score`: fires only when a player knows exactly
    /// what a card is, enabling confident play or discard decisions.
    fn resolved_card_count(
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let mut total = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                let card_deck_index = idx as CardDeckIndex;
                let combined = pk.combined_possible_identities(
                    card_deck_index,
                    table_state,
                    &static_data.variant,
                );
                if combined.is_exactly_known() {
                    total += 1.0;
                }
            }
        }
        total
    }

    /// Count of known-playable cards held by the player behind `pov`.
    ///
    /// Delegates to [`Self::card_known_playable`] for each card in the player's hand so the
    /// definition of "known playable" is consistent across all evaluator terms.
    fn count_known_playable(pov: &dyn PlayerPOV, playable_mask: VariantCardsBitField) -> u32 {
        let pk = pov.team_knowledge().player(pov.player_index());
        let mut hand = pk.own_hand;
        let mut count = 0u32;
        while hand != 0 {
            let idx = hand.trailing_zeros() as CardDeckIndex;
            hand &= hand - 1;
            if Self::card_known_playable(pov, idx, playable_mask) {
                count += 1;
            }
        }
        count
    }
}

impl Evaluator for DefaultEvaluator {
    fn score(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> Score {
        self.score_breakdown(state, truth, phantom_plays).total
    }

    fn score_breakdown(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> ScoreBreakdown {
        let table_state = state.table_state();
        let static_data = state.static_data();
        let team_knowledge = state.team_knowledge();
        let game_score =
            self.score_weight * state.score(&static_data.variant, phantom_plays) as f64;
        let strikes = table_state.strike_tokens as usize;
        let strike_penalty = self.strike_penalties.get(strikes).copied().unwrap_or(0.0);
        if table_state.is_terminal(static_data) {
            return ScoreBreakdown {
                game_score,
                strike_penalty,
                pace: 0.0,
                efficiency_penalty: 0.0,
                critical_exposure_penalty: 0.0,
                lost_ceiling_penalty: 0.0,
                empathy_bonus: 0.0,
                clue_tokens: 0.0,
                known_playable: 0.0,
                team_empathy: 0.0,
                misinformation_penalty: 0.0,
                critical_exposure_delta: 0.0,
                total: game_score - strike_penalty,
            };
        }
        let pace = self.pace_weight
            * (state.pace(phantom_plays)).clamp(-10, static_data.number_of_players as i32) as f64;
        let efficiency_penalty =
            self.efficiency_weight * f64::from(state.required_efficiency(phantom_plays));
        let critical_exposure_penalty = if self.critical_exposure_weight != 0.0 {
            self.critical_exposure_weight
                * Self::critical_exposure_score(table_state, static_data, team_knowledge, truth)
        } else {
            0.0
        };
        let theoretical_max =
            (static_data.variant.number_of_suits * static_data.variant.stacks_size) as f64;
        let lost_ceiling_penalty = self.lost_score_ceiling_weight
            * (theoretical_max - table_state.max_achievable_score(static_data) as f64);
        let empathy_bonus = if self.empathy_weight != 0.0 {
            self.empathy_weight * Self::empathy_precision(table_state, static_data, team_knowledge)
        } else {
            0.0
        };
        let harmonic_value = Self::harmonic(table_state.clue_token_bank.whole_clue_tokens_count());
        let demand_factor = if self.clue_demand_weight != 0.0 {
            1.0 + self.clue_demand_weight * Self::clue_demand(table_state, static_data)
        } else {
            1.0
        };
        let clue_tokens = self.clue_token_weight * harmonic_value * demand_factor;
        let known_playable = if self.known_playable_weight != 0.0 {
            self.known_playable_weight
                * Self::known_playable_in_hands(table_state, static_data, team_knowledge, truth)
        } else {
            0.0
        };
        // team_empathy is no longer a leaf term — it accumulates through clue actions via
        // `team_empathy_delta_bonus`. The breakdown field is kept (zeroed here) so callers
        // that aggregate per-line bonuses can record what the clue deltas contributed.
        let team_empathy = 0.0;
        let misinformation_penalty = if self.misinformation_weight != 0.0 {
            self.misinformation_weight
                * Self::misinformation_score(static_data, team_knowledge, table_state, truth)
        } else {
            0.0
        };
        let total = game_score - strike_penalty + pace
            - efficiency_penalty
            - critical_exposure_penalty
            - lost_ceiling_penalty
            + empathy_bonus
            + clue_tokens
            + known_playable
            - misinformation_penalty;
        ScoreBreakdown {
            game_score,
            strike_penalty,
            pace,
            efficiency_penalty,
            critical_exposure_penalty,
            lost_ceiling_penalty,
            empathy_bonus,
            clue_tokens,
            known_playable,
            team_empathy,
            misinformation_penalty,
            critical_exposure_delta: 0.0,
            total,
        }
    }

    fn clue_precision_bonus(
        &self,
        touched: &[u8],
        receiver: usize,
        giver: usize,
        truth: &dyn PlayerPOV,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> Score {
        let precision_bonus = if self.clue_precision_weight != 0.0 {
            touched
                .iter()
                .filter(|&&idx| {
                    let card_deck_index = idx as CardDeckIndex;
                    let combined = team_knowledge
                        .player(receiver)
                        .combined_possible_identities(
                            card_deck_index,
                            table_state,
                            &static_data.variant,
                        );
                    combined.is_exactly_known()
                })
                .count() as f64
                * self.clue_precision_weight
        } else {
            0.0
        };

        let bad_touch_count = count_bad_touches(touched, receiver, truth, table_state, static_data);

        let potential_bad_touch = self.potential_bad_touch_penalty != 0.0
            && is_potential_bad_touch(
                touched,
                giver,
                truth,
                table_state,
                static_data,
                team_knowledge,
            );

        precision_bonus
            - bad_touch_count as f64 * self.good_touch_penalty
            - if potential_bad_touch {
                self.potential_bad_touch_penalty
            } else {
                0.0
            }
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
            GameAction::Play {
                card_deck_index, ..
            } => Some(*card_deck_index),
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
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
        pre_phantom_plays: u8,
        post_phantom_plays: u8,
    ) -> Score {
        if self.play_progress_weight == 0.0 {
            return 0.0;
        }
        let GameAction::Play { .. } = action else {
            return 0.0;
        };
        let variant = &pre.static_data().variant;
        if post.score(variant, post_phantom_plays) > pre.score(variant, pre_phantom_plays) {
            self.play_progress_weight
        } else {
            0.0
        }
    }

    fn discard_action_penalty(
        &self,
        action: &GameAction,
        actor: PlayerIndex,
        pre: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
    ) -> Score {
        let GameAction::Discard {
            card_deck_index, ..
        } = action
        else {
            return 0.0;
        };
        let table_state = pre.table_state();
        let static_data = pre.static_data();
        let bdr = if self.bottom_deck_risk_weight != 0.0 {
            self.bottom_deck_risk_weight
                * Self::bottom_deck_risk_score(*card_deck_index, table_state, static_data, truth)
        } else {
            0.0
        };
        let kp_penalty = if self.discard_while_known_playable_penalty != 0.0
            && Self::actor_has_known_playable(actor, table_state, static_data, pre.team_knowledge())
        {
            self.discard_while_known_playable_penalty
        } else {
            0.0
        };
        -(bdr + kp_penalty)
    }

    fn team_empathy_delta_bonus(
        &self,
        action: &GameAction,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
    ) -> Score {
        if self.team_empathy_weight == 0.0 {
            return 0.0;
        }
        let GameAction::Clue { .. } = action else {
            return 0.0;
        };
        let static_data = pre.static_data();
        let pre_score =
            Self::team_empathy_score(static_data, pre.team_knowledge(), pre.table_state());
        let post_score =
            Self::team_empathy_score(static_data, post.team_knowledge(), post.table_state());
        self.team_empathy_weight * (post_score - pre_score)
    }

    fn critical_exposure_delta_bonus(
        &self,
        _action: &GameAction,
        actor: PlayerIndex,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
    ) -> Score {
        if self.critical_exposure_delta_weight == 0.0 {
            return 0.0;
        }
        let pre_table = pre.table_state();
        let post_table = post.table_state();
        let static_data = pre.static_data();
        let num_players = static_data.number_of_players as usize;

        let pre_critical_mask = Self::critical_mask(pre_table, static_data);
        if pre_critical_mask == 0 {
            return 0.0;
        }

        let save_available = pre_table.clue_token_bank.whole_clue_tokens_count() > 0;
        // Only compute post mask when needed (save_available = true, card still exposed).
        let post_critical_mask = if save_available {
            Self::critical_mask(post_table, static_data)
        } else {
            0
        };

        let playable_mask = pre_table.playable_cards(static_data);
        let actor_pk = pre.team_knowledge().player(actor);
        let actor_pov =
            LightweightPlayerPOV::new(actor, actor_pk, pre.team_knowledge(), pre_table, static_data);
        let actor_known_playable = Self::count_known_playable(&actor_pov, playable_mask);

        let mut total = 0.0f64;
        for t in 0..num_players {
            if t == actor {
                continue;
            }
            let pk = pre.team_knowledge().player(t);
            let pov = LightweightPlayerPOV::new(t, pk, pre.team_knowledge(), pre_table, static_data);
            let Some(chop_idx) = get_chop_index(t, &pov) else {
                continue;
            };
            let Some(chop_id) = truth.card_identity(chop_idx) else {
                continue; // truth cannot see this card — skip
            };
            if (1u64 << chop_id) & pre_critical_mask == 0 {
                continue; // not truth-critical
            }
            let saved = (post_table.clue_touched_cards >> chop_idx) & 1 != 0;
            if saved {
                total += self.critical_exposure_delta_weight;
            } else if save_available && (1u64 << chop_id) & post_critical_mask != 0 {
                // Charge the actor only if no player acting between actor and t has strictly
                // fewer known playables (a player with fewer known playables is "freer" to save
                // and is the more natural saver).
                let any_freer_intermediate = {
                    let mut p = (actor + 1) % num_players;
                    let mut found = false;
                    while p != t {
                        let int_pk = pre.team_knowledge().player(p);
                        let int_pov = LightweightPlayerPOV::new(
                            p,
                            int_pk,
                            pre.team_knowledge(),
                            pre_table,
                            static_data,
                        );
                        if Self::count_known_playable(&int_pov, playable_mask)
                            < actor_known_playable
                        {
                            found = true;
                            break;
                        }
                        p = (p + 1) % num_players;
                    }
                    found
                };
                if !any_freer_intermediate {
                    total -= self.critical_exposure_delta_weight;
                }
            }
        }
        total
    }

    fn upper_bound(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        depth: usize,
    ) -> Score {
        let max_game_score =
            self.score_weight * table_state.max_achievable_score(static_data) as f64;
        // Optimistic: no new strikes from here; penalty is already locked in at current level.
        let min_strike_penalty = self
            .strike_penalties
            .get(table_state.strike_tokens as usize)
            .copied()
            .unwrap_or(0.0);
        let max_pace = self.pace_weight * static_data.number_of_players as f64;
        let num_players = static_data.number_of_players as usize;
        let total_cards: usize = table_state.hands[..num_players]
            .iter()
            .map(|h| h.cards().len())
            .sum();
        // Optimistic: max clue tokens with max demand factor.
        let max_demand = if self.clue_demand_weight > 0.0 {
            1.0 + self.clue_demand_weight * total_cards as f64
        } else {
            1.0
        };
        let max_clue_tokens =
            self.clue_token_weight * Self::harmonic(crate::game::MAX_CLUE_TOKEN_COUNT) * max_demand;
        let max_known_playable = self.known_playable_weight * total_cards as f64;
        // Optimistic: efficiency_penalty = 0, lost_ceiling_penalty = 0, misinformation = 0,
        // critical_exposure_penalty = 0 (no critical cards in any hand).
        let max_leaf =
            max_game_score - min_strike_penalty + max_pace + max_clue_tokens + max_known_playable;
        // Immediate bonuses: per-ply max = play_progress + clue precision across all cards
        // + team-empathy delta (each clue can at most lift the whole-hand score by total_cards)
        // + critical_exposure_delta (at most one reward per teammate = num_players - 1).
        let max_immediate_per_ply = self.play_progress_weight
            + self.clue_precision_weight * total_cards as f64
            + self.team_empathy_weight * total_cards as f64
            + self.critical_exposure_delta_weight
                * (static_data.number_of_players as f64 - 1.0).max(0.0);
        max_leaf + max_immediate_per_ply * depth as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
    use crate::game::clue_token_bank::ClueTokenBank;
    use crate::game::deck::Deck;
    use crate::game::hand::Hand;
    use crate::game::playing_stacks::PlayingStacks;
    use crate::game::state::table_state::TableState;
    use crate::game::static_game_data::StaticGameData;
    use crate::game::variant::test_variants::NO_VARIANT;

    /// Build a truth POV for tests by holding owned values and exposing them by reference.
    /// `card_identity` falls back to the deck's public empathy via `combined_possible_identities`.
    struct TruthFixture {
        knowledge: PlayerKnowledge,
        team_knowledge: TeamKnowledge,
    }

    impl TruthFixture {
        fn new(static_data: &StaticGameData) -> Self {
            TruthFixture {
                knowledge: PlayerKnowledge::new(0),
                team_knowledge: TeamKnowledge::new(static_data.number_of_players as usize),
            }
        }

        fn pov<'a>(
            &'a self,
            table_state: &'a TableState,
            static_data: &'a StaticGameData,
        ) -> LightweightPlayerPOV<'a> {
            // Omniscient testing POV: pretend every card is directly visible so
            // `card_identity` falls back to deck-empathy truth.
            LightweightPlayerPOV::with_visible_cards(
                0,
                &self.knowledge,
                &self.team_knowledge,
                table_state,
                static_data,
                u64::MAX,
            )
        }
    }

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

    fn make_kags(strikes: u8) -> KnowledgeAwareGameState {
        let (table_state, static_data) = make_state(strikes);
        let tk = TeamKnowledge::new(static_data.number_of_players as usize);
        KnowledgeAwareGameState::from_parts(static_data, table_state, tk, 0)
    }

    #[test]
    fn higher_strikes_produce_lower_score() {
        let evaluator = DefaultEvaluator::default();
        let s0 = make_kags(0);
        let s1 = make_kags(1);
        let s2 = make_kags(2);
        let truth = TruthFixture::new(s0.static_data());
        let p0 = truth.pov(s0.table_state(), s0.static_data());
        let p1 = truth.pov(s1.table_state(), s1.static_data());
        let p2 = truth.pov(s2.table_state(), s2.static_data());
        assert!(evaluator.score(&s0, &p0, 0) > evaluator.score(&s1, &p1, 0));
        assert!(evaluator.score(&s1, &p1, 0) > evaluator.score(&s2, &p2, 0));
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
        });

        // Discarding while signalled → full penalty.
        let discard = GameAction::Discard {
            player_index: 0,
            card_deck_index: 5,
            turn: 1,
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
            turn: 1,
        };
        let pen = evaluator.signal_ignored_penalty(&clue, 0, &static_data, &tk, &table_state);
        assert_eq!(pen, -evaluator.signal_ignored_penalty_weight);

        // Playing the signalled card → no penalty.
        let play = GameAction::Play {
            player_index: 0,
            card_deck_index: 5,
            turn: 1,
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
            turn: 1,
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
        });

        let discard = GameAction::Discard {
            player_index: 0,
            card_deck_index: 5,
            turn: 1,
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
        deck.reveal_card(5, 7, &NO_VARIANT); // truth = card 7 at position 5

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

        let truth_fixture = TruthFixture::new(&static_data);
        let truth = truth_fixture.pov(&state, &static_data);
        let score_misinformed =
            DefaultEvaluator::misinformation_score(&static_data, &tk, &state, &truth);
        assert!(
            score_misinformed > 0.0,
            "misinformation_score should be positive when effective mask excludes truth (got {score_misinformed})"
        );

        // Correct the knowledge: effective mask now includes the truth.
        tk.player_mut(0).inferred_identities[5] =
            Some(crate::game::card::CardIdentityMask::from_bits(1 << 7));
        let score_correct =
            DefaultEvaluator::misinformation_score(&static_data, &tk, &state, &truth);
        assert_eq!(
            score_correct, 0.0,
            "misinformation_score should be 0 when knowledge exactly matches truth"
        );
    }

    /// Build a table state with a single hand-size-5 hand in player 0, whose deck slot 4
    /// is card-id `id_at_chop` (the chop card under H-Group's rightmost-untouched rule).
    /// All cards live at deck indices [0..5]. Truth identities are written via `reveal_card`.
    fn make_chop_test_state(ids_in_hand: [u8; 5]) -> (TableState, StaticGameData, TeamKnowledge) {
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let mut deck = Deck::new(&NO_VARIANT);
        for (slot, &id) in ids_in_hand.iter().enumerate() {
            deck.reveal_card(slot as u8, id as usize, &NO_VARIANT);
        }
        let mut hands = Hand::empty_array();
        hands[0] = Hand::new(&[0, 1, 2, 3, 4]);
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let mut tk = TeamKnowledge::new(3);
        *tk.player_mut(0) = knowledge_for_hand(&[0, 1, 2, 3, 4]);
        (state, static_data, tk)
    }

    /// Confirms the formula `1 / (1 + r + 4*b)` for the chop, no-buffer case:
    /// a single truth-critical card on chop with no buffer in hand → threat = 1.0.
    /// `Hand::new` takes oldest-first input, so array index 0 = chop, index 4 = slot 1.
    /// Non-critical fillers are rank-3 cards: not playable from empty stacks (avoiding
    /// the known-playable buffer effect) and not critical (2 copies, 0 discarded).
    #[test]
    fn critical_exposure_chop_no_buffer_is_one() {
        // [chop=R5, Y3, G3, B3, slot1=P3]
        let (state, static_data, tk) = make_chop_test_state([4, 7, 12, 17, 22]);
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        assert!(
            (score - 1.0).abs() < 1e-9,
            "chop critical with no buffer should be exactly 1.0, got {score}"
        );
    }

    /// Moving the critical card from chop to slot 1 (newest) reduces threat to
    /// 1 / (1 + 4 + 0) = 0.2 (4 positions away from chop, no buffer).
    #[test]
    fn critical_exposure_slot1_lower_than_chop() {
        // [chop=Y3, G3, B3, P3, slot1=R5]
        let (state, static_data, tk) = make_chop_test_state([7, 12, 17, 22, 4]);
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        let expected = 1.0 / (1.0 + 4.0);
        assert!(
            (score - expected).abs() < 1e-9,
            "slot-1 critical with no buffer should be {expected}, got {score}"
        );
    }

    /// Buffer dominates position (factor 4). Four known-playables in hand pushes the
    /// chop critical's threat to 1 / (1 + 0 + 16) ≈ 0.059 — much smaller than even the
    /// furthest no-buffer slot (0.2).
    #[test]
    fn critical_exposure_buffer_dominates_position() {
        // [chop=R5, R1, Y1, G1, slot1=B1]. Each rank-1 is convention-narrowed to its
        // singleton identity (modeling a clue that resolved the card to a known
        // playable), so buffer = 4. R5 stays chop-eligible.
        let (mut state, static_data, tk) = make_chop_test_state([4, 0, 5, 10, 15]);
        let mut tk = tk;
        for (slot, id) in [(1, 0), (2, 5), (3, 10), (4, 15)] {
            tk.player_mut(0).inferred_identities[slot] =
                Some(crate::game::card::CardIdentityMask::from_bits(1 << id));
            state.clue_touched_cards |= 1u64 << slot;
        }
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        let expected = 1.0 / (1.0 + 0.0 + 4.0 * 4.0);
        assert!(
            (score - expected).abs() < 1e-9,
            "chop critical with 4 buffer should be {expected}, got {score}"
        );
    }

    /// When no critical mask exists (no card has remaining == 1), the term is zero.
    #[test]
    fn critical_exposure_zero_when_no_critical_mask() {
        // Hand: [R1, Y1, G1, B1, P1]. None of these are critical at game start
        // (each rank-1 has 3 copies, 0 discarded → remaining = 3).
        let (state, static_data, tk) = make_chop_test_state([0, 5, 10, 15, 20]);
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        assert_eq!(score, 0.0, "no critical identities → zero exposure");
    }

    /// A touched (non-playable) critical card contributes zero — it is held indefinitely
    /// and is not on the discard path.
    #[test]
    fn critical_exposure_zero_when_critical_is_touched() {
        // [chop=R5, Y3, G3, B3, slot1=P3] — R5 at deck index 0.
        let (mut state, static_data, tk) = make_chop_test_state([4, 7, 12, 17, 22]);
        state.clue_touched_cards |= 1u64 << 0;
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        assert_eq!(
            score, 0.0,
            "touched non-playable critical should not contribute to exposure"
        );
    }

    /// With a full deck (50 cards) and maximum empathy (25 ids, 15 BDR-eligible), the
    /// bottom deck risk on a single discard must be small — Term B (1/50) dominates.
    #[test]
    fn bdr_full_deck_is_small() {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        // Deck at full size; card at deck index 0 is unrevealed (full empathy, 25 ids).
        let deck = Deck::new(&NO_VARIANT);
        let hands = Hand::empty_array();
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
        assert!(
            bdr < 0.5,
            "full-deck BDR should be small (got {bdr}); Term B = 1/50 should dominate"
        );
    }

    /// With only 1 card left in the deck, Term B = 1.0 and BDR is at its maximum value —
    /// the same state that was safe with a full deck is now a genuine ceiling-loss risk.
    #[test]
    fn bdr_empty_deck_is_large() {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let mut deck = Deck::new(&NO_VARIANT);
        deck.current_size = 1;
        let hands = Hand::empty_array();
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
        // With full deck the value is ~0.036; with 1-card deck it should be ~50× larger (~1.8).
        assert!(
            bdr > 1.0,
            "near-empty-deck BDR should be large (got {bdr}); Term B = 1/1 = 1.0"
        );
    }

    /// When the surviving copy is visibly held by another player, BDR must be zero —
    /// the `visible_ids` short-circuit eliminates any chance of the card being lost.
    #[test]
    fn bdr_zero_when_surviving_copy_visible() {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        // Reveal deck index 0 AND deck index 1 as Red 2 (id = 1, the two copies of that card).
        let mut deck = Deck::new(&NO_VARIANT);
        deck.reveal_card(0, 1, &NO_VARIANT); // discarded card → truth id 1
        deck.reveal_card(1, 1, &NO_VARIANT); // surviving copy visible in another hand
        let mut hands = Hand::empty_array();
        hands[1] = Hand::new(&[1]); // player 1 holds deck index 1
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
        assert_eq!(bdr, 0.0, "visible surviving copy must zero out BDR");
    }

    /// BDR penalty from `discard_action_penalty` scales linearly with `bottom_deck_risk_weight`.
    #[test]
    fn bdr_scales_linearly_with_weight() {
        use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let deck = Deck::new(&NO_VARIANT);
        let hands = Hand::empty_array();
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let tk = TeamKnowledge::new(static_data.number_of_players as usize);
        let kags = KnowledgeAwareGameState::from_parts(static_data, state, tk, 0);
        let truth = TruthFixture::new(kags.static_data());
        let truth_pov = truth.pov(kags.table_state(), kags.static_data());
        let action = GameAction::Discard {
            player_index: 0,
            card_deck_index: 0,
            turn: 1,
        };
        let w1 = DefaultEvaluator {
            bottom_deck_risk_weight: 0.1,
            ..DefaultEvaluator::default()
        };
        let w2 = DefaultEvaluator {
            bottom_deck_risk_weight: 0.5,
            ..DefaultEvaluator::default()
        };
        // Disable the known-playable penalty to isolate BDR.
        let w1 = DefaultEvaluator {
            discard_while_known_playable_penalty: 0.0,
            ..w1
        };
        let w2 = DefaultEvaluator {
            discard_while_known_playable_penalty: 0.0,
            ..w2
        };
        let p1 = w1.discard_action_penalty(&action, 0, &kags, &truth_pov);
        let p2 = w2.discard_action_penalty(&action, 0, &kags, &truth_pov);
        // Both penalties are non-positive; ratio must equal weight ratio.
        assert!(p1 < 0.0 && p2 < 0.0, "BDR penalty must be negative");
        let ratio = p2 / p1;
        let expected = 0.5 / 0.1;
        assert!(
            (ratio - expected).abs() < 1e-9,
            "penalty ratio {ratio} should equal weight ratio {expected}"
        );
    }

    /// A chop card with no truth identity (freshly drawn in search — not revealed in the
    /// deck) must contribute zero even though its full empathy spans critical identities.
    /// Probabilistic risk from unknown cards belongs to BDR, not this term.
    #[test]
    fn critical_exposure_zero_for_unresolved_chop() {
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let mut deck = Deck::new(&NO_VARIANT);
        // Deck index 0 (chop) is intentionally left unrevealed — truth.card_identity(0)
        // returns None because the global empathy has multiple bits. Deck indices 1-4 get
        // non-critical fillers so they don't interfere with the assertion.
        for (offset, &id) in [7u8, 12, 17, 22].iter().enumerate() {
            deck.reveal_card((offset + 1) as u8, id as usize, &NO_VARIANT);
        }
        let mut hands = Hand::empty_array();
        hands[0] = Hand::new(&[0, 1, 2, 3, 4]);
        let state = TableState::from_parts(
            ClueTokenBank::new(10),
            deck,
            hands,
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let mut tk = TeamKnowledge::new(3);
        *tk.player_mut(0) = knowledge_for_hand(&[0, 1, 2, 3, 4]);
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let score =
            DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
        assert_eq!(
            score, 0.0,
            "unresolved chop (no truth identity) must not contribute to critical exposure"
        );
    }

    // ── critical_exposure_delta_bonus ───────────────────────────────────────────

    /// Build a two-player (actor=0, teammate=1) KAGS where player 1 has a truth-critical card
    /// at deck index 10 on their chop. Returns (pre_state, post_state_unsaved,
    /// post_state_saved).
    ///
    /// `with_clue_tokens`: whether actor has clue tokens.
    fn make_critical_chop_states(
        with_clue_tokens: bool,
    ) -> (
        KnowledgeAwareGameState,
        KnowledgeAwareGameState,
        KnowledgeAwareGameState,
    ) {
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
        use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;

        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        // Card id 4 = R5, only 1 copy → always critical.
        let mut deck = Deck::new(&NO_VARIANT);
        deck.reveal_card(10, 4, &NO_VARIANT); // deck index 10 = R5
        let mut hands = Hand::empty_array();
        hands[0] = Hand::new(&[0]); // actor (player 0)
        hands[1] = Hand::new(&[10]); // teammate (player 1), chop = deck 10
        let clue_tokens = if with_clue_tokens { 8 } else { 0 };
        let pre_state = TableState::from_parts(
            ClueTokenBank::new(clue_tokens),
            deck.clone(),
            hands.clone(),
            0,
            1,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let mut tk = TeamKnowledge::new(2);
        *tk.player_mut(0) = knowledge_for_hand(&[0]);
        *tk.player_mut(1) = knowledge_for_hand(&[10]);
        let pre = KnowledgeAwareGameState::from_parts(static_data.clone(), pre_state.clone(), tk.clone(), 11);

        // post_unsaved: same table state (actor did not touch the chop card)
        let post_unsaved = KnowledgeAwareGameState::from_parts(static_data.clone(), pre_state.clone(), tk.clone(), 11);

        // post_saved: chop card is now clue-touched
        let mut saved_state = pre_state.clone();
        saved_state.clue_touched_cards |= 1u64 << 10;
        let post_saved = KnowledgeAwareGameState::from_parts(static_data, saved_state, tk, 11);

        (pre, post_unsaved, post_saved)
    }

    #[test]
    fn critical_exposure_delta_penalty_when_save_skipped() {
        let (pre, post_unsaved, _) = make_critical_chop_states(true);
        let evaluator = DefaultEvaluator {
            critical_exposure_delta_weight: 2.0,
            ..DefaultEvaluator::default()
        };
        let truth = TruthFixture::new(pre.static_data());
        let truth_pov = truth.pov(pre.table_state(), pre.static_data());
        let action = GameAction::Discard {
            player_index: 0,
            card_deck_index: 0,
            turn: 1,
        };
        let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_unsaved, &truth_pov);
        assert!(
            delta < 0.0,
            "should charge when critical chop left unsaved with tokens available (got {delta})"
        );
        assert!(
            (delta - (-2.0)).abs() < 1e-9,
            "penalty should equal -weight (got {delta})"
        );
    }

    #[test]
    fn critical_exposure_delta_reward_when_chop_saved() {
        let (pre, _, post_saved) = make_critical_chop_states(true);
        let evaluator = DefaultEvaluator {
            critical_exposure_delta_weight: 2.0,
            ..DefaultEvaluator::default()
        };
        let truth = TruthFixture::new(pre.static_data());
        let truth_pov = truth.pov(pre.table_state(), pre.static_data());
        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: crate::game::clue::Clue {
                clue_type: crate::game::clue_type::ClueType::Rank,
                clue_value: 5,
            },
            turn: 1,
        };
        let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_saved, &truth_pov);
        assert!(
            delta > 0.0,
            "should reward when critical chop card is saved (got {delta})"
        );
        assert!(
            (delta - 2.0).abs() < 1e-9,
            "reward should equal +weight (got {delta})"
        );
    }

    #[test]
    fn critical_exposure_delta_no_penalty_when_no_clue_tokens() {
        let (pre, post_unsaved, _) = make_critical_chop_states(false);
        let evaluator = DefaultEvaluator {
            critical_exposure_delta_weight: 2.0,
            ..DefaultEvaluator::default()
        };
        let truth = TruthFixture::new(pre.static_data());
        let truth_pov = truth.pov(pre.table_state(), pre.static_data());
        let action = GameAction::Discard {
            player_index: 0,
            card_deck_index: 0,
            turn: 1,
        };
        let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_unsaved, &truth_pov);
        assert_eq!(
            delta, 0.0,
            "no penalty when no clue tokens available (got {delta})"
        );
    }

    #[test]
    fn critical_exposure_delta_zero_when_no_critical_cards() {
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
        use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;

        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        // Card id 0 = R1, 3 copies → not critical.
        let mut deck = Deck::new(&NO_VARIANT);
        deck.reveal_card(10, 0, &NO_VARIANT);
        let mut hands = Hand::empty_array();
        hands[0] = Hand::new(&[0]);
        hands[1] = Hand::new(&[10]);
        let state = TableState::from_parts(
            ClueTokenBank::new(8),
            deck,
            hands,
            0,
            1,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        );
        let mut tk = TeamKnowledge::new(2);
        *tk.player_mut(0) = knowledge_for_hand(&[0]);
        *tk.player_mut(1) = knowledge_for_hand(&[10]);
        let kags = KnowledgeAwareGameState::from_parts(static_data.clone(), state.clone(), tk, 11);
        let evaluator = DefaultEvaluator {
            critical_exposure_delta_weight: 2.0,
            ..DefaultEvaluator::default()
        };
        let truth = TruthFixture::new(&static_data);
        let truth_pov = truth.pov(&state, &static_data);
        let action = GameAction::Discard {
            player_index: 0,
            card_deck_index: 0,
            turn: 1,
        };
        let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &kags, &kags, &truth_pov);
        assert_eq!(delta, 0.0, "no effect when no critical cards exist (got {delta})");
    }
}
