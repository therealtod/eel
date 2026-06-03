use crate::engine::convention::hgroup::h_group_core::{
    get_chop_index, is_save_principle_protected, still_needed_cards_mask,
};
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::evaluator::types::{HARMONIC_LUT, RECIPROCAL_LUT};
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

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
///                                                        cards (good-touch principle violation). Each penalty is softened by
///                                                        `forced_save_bad_touch_relief` when the clue is a forced
///                                                        Save-Principle save (see `forced_save_bad_touch_relief`).
/// - `-potential_bad_touch_penalty`                     — flat penalty when a touched-on-receiver still-needed
///                                                        identity overlaps with an identity the giver likely
///                                                        already holds (potential duplicate)
///
/// Per-ply immediate adjustments (applied to every action along the search line):
/// - `-tempo_cost_weight`                               — flat penalty for any non-play turn (clues, discards,
///                                                        misplays); closes the horizon-arbitrage hole where the
///                                                        engine pads lines with redundant clues to push discards
///                                                        past the leaf
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
    /// card held in a player's hand is `1 / (1 + rank + 4 * (buffer_count + touched_shield))`,
    /// where:
    ///
    /// - `rank` is the number of untouched chop-eligible cards in front of the critical (closer
    ///   to chop), i.e. its position in the holder's discard order: 0 if it is the current chop,
    ///   1 if one untouched card sits ahead of it, etc. Each only pushes the critical one
    ///   position away (×1).
    /// - `buffer_count` is the number of known-playables plus known-trash cards in the holder's
    ///   hand. Each one represents an action (play or trash-discard) the holder will take before
    ///   being forced to discard the critical card.
    /// - `touched_shield` is the number of *touched, non-playable, non-critical* cards in front
    ///   of the critical. These are held indefinitely (never discarded), so saving such a card
    ///   protects the critical — and, crucially, the term must not *penalise* the save: a card
    ///   going untouched → touched would otherwise just leave the chop-eligible run and bump the
    ///   critical one rank closer to chop.
    ///
    /// Buffer/shield dominates position (factor 4): one safe play/trash-discard or one held
    /// non-critical card in front ≈ moving the critical four slots away from chop, so even a
    /// well-shielded critical near chop is safer than an unbuffered critical sitting on chop.
    ///
    /// Only cards *in front of* the critical protect it; cards *behind* it (further from chop)
    /// are reached only after it and contribute nothing.
    ///
    /// Cards classified as **touched but not known-playable** (clued saves, prompts, …) are
    /// themselves never at risk — they contribute zero exposure of their own — but a non-critical
    /// one shields the criticals behind it. Touched + known-playable counts as buffer.
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
    /// Positive **relief** subtracted from [`Self::good_touch_penalty`] for each bad touch that is
    /// *forced by a Save-Principle save* — a clue that saves a protected card sitting on the
    /// receiver's chop, with the chop card touched (see `h_group_core::is_forced_save`). The
    /// effective per-bad-touch penalty becomes `(good_touch_penalty - forced_save_bad_touch_relief)`
    /// (floored at 0).
    ///
    /// The Save Principle outranks the Good Touch Principle: when saving a protected card on chop
    /// cannot avoid touching collateral trash, the bad touch is *forced by circumstance*, not
    /// chosen, so it must not be penalised as harshly as a deliberate one — otherwise the engine
    /// would rather let a protected card be discarded than incur the full good-touch penalty.
    ///
    /// Expressed as a relief (rather than a separate absolute penalty) so it tracks
    /// `good_touch_penalty` under re-tuning and can never exceed it. Kept **below**
    /// `good_touch_penalty` so the softened penalty stays **non-zero**: among competing saves of
    /// the same card, search still prefers one that avoids the bad touch (e.g. a clean rank save
    /// over a colour save that also touches trash). Suggested default: `15.0` (→ effective `5.0`).
    ///
    /// Set to `0.0` to disable the relief (full good-touch penalty even for forced saves).
    pub forced_save_bad_touch_relief: f64,
    /// Positive **relief** subtracted from [`Self::good_touch_penalty`] for each bad touch on a clue
    /// that is *forced by a stall situation* — the clue-token bank is full
    /// ([`MAX_CLUE_TOKEN_COUNT`](crate::game::MAX_CLUE_TOKEN_COUNT)), so the giver cannot discard and
    /// must give *some* clue. The effective per-bad-touch penalty becomes
    /// `(good_touch_penalty - forced_stall_bad_touch_relief)` (floored at 0).
    ///
    /// Same rationale and shape as [`Self::forced_save_bad_touch_relief`]: a bad touch incurred only
    /// because the rules leave no legal discard is *forced by circumstance*, not chosen, so it must
    /// not be penalised as harshly as a deliberate one. Crucially, the relief is gated on the bank
    /// actually being full, so a player who can *still discard* pays the **full** good-touch penalty
    /// for a bad-touch clue — and therefore prefers to discard now and defer such a clue until it is
    /// genuinely forced. This is the deferral that keeps the engine from cluing trash early when a
    /// clean discard is available.
    ///
    /// Kept **below** `good_touch_penalty` so the softened penalty stays **non-zero**: even when
    /// forced to clue, a stall clue that bad-touches less is still preferred. Suggested default:
    /// `15.0` (→ effective `5.0`).
    ///
    /// Set to `0.0` to disable the relief (full good-touch penalty even for forced stalls).
    pub forced_stall_bad_touch_relief: f64,
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
    /// Per-card standing penalty for **parked dead weight** — clue-touched cards whose true
    /// identity is already dead (not still-needed) but whose holder still believes them
    /// potentially useful, so they sit in hand instead of being discarded. See
    /// [`Self::parked_dead_weight`].
    ///
    /// This is the leaf-resident dual of [`Self::good_touch_penalty`] (which is charged once,
    /// at clue time). Where `good_touch_penalty` punishes the *act* of a bad touch, this term
    /// punishes the *standing state* of duplicates left parked in a hand, so the search keeps
    /// paying for them across the rollout rather than recouping the one-time charge with
    /// optimistic plays the short horizon can't refute.
    ///
    /// Set to 0 to disable.
    pub parked_dead_weight_penalty: f64,
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
    /// `P(card_is_id) × loss_if_lost(id)` where:
    ///
    /// - `P(card_is_id)` = `1 / popcount` — uniform over empathy (Term A).
    /// - `loss_if_lost` = achievable ranks from `rank_idx` upward, stopping at the first
    ///   rank already fully in the discard pile (Term C).
    ///
    /// The charge is **deck-position-independent**: a discard that turns a card critical costs the
    /// same whether the deck is full or nearly empty (no Laplace `1 / deck_remaining` factor). See
    /// [`Self::bottom_deck_risk_score`] for why that probabilistic discount was removed.
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
    /// being dumped may itself be critical.
    ///
    /// Sized at [`Self::score_weight`] (the value of one realized play): discarding while
    /// holding a guaranteed play forgoes a full point's worth of progress, so the penalty must
    /// at least match that to keep the play preferred even when a deep-rollout artifact (e.g. a
    /// POV-restricted teammate forced into a bad-touch save) inflates the play line's apparent
    /// cost. Below `score_weight`, such artifacts can flip a clearly-correct known play to a chop
    /// discard (see the `should_play_instead_of_discarding` replay regression).
    ///
    /// Softened by [`Self::save_pressure_known_playable_relief`] when the team is under save
    /// pressure (a protected card is about to be discarded), excusing the actor for using the
    /// turn on the save instead of cashing their play.
    ///
    /// Set to 0 to disable.
    pub discard_while_known_playable_penalty: f64,
    /// Positive **relief** subtracted from [`Self::discard_while_known_playable_penalty`] when the
    /// actor — though holding a known-playable — is under *Save-Principle pressure*: another
    /// player's unclued chop holds a protected card ([`is_save_principle_protected`]) and that
    /// holder has no known-playable of their own, so the card is in concrete danger of being
    /// discarded. The effective penalty becomes
    /// `(discard_while_known_playable_penalty - save_pressure_known_playable_relief)` (floored at 0).
    ///
    /// The Save Principle outranks the team's tempo expectation that a player always cashes a known
    /// play: when a protected card is about to be lost, the team is in "save mode", and a player
    /// who spends the turn helping the save (e.g. cycling for a clue token) rather than cashing
    /// their own play should not be charged the full "you sat on a known play" penalty. This is the
    /// deferral-side dual of [`Self::forced_save_bad_touch_relief`], and uses the same relief shape
    /// for consistency.
    ///
    /// This is also self-limiting against the artifact the base penalty guards: the same save now
    /// costs less in the play line's rollout thanks to `forced_save_bad_touch_relief`, so softening
    /// here does not reopen the discard-over-play hole. Kept **below**
    /// `discard_while_known_playable_penalty` so the softened penalty stays non-zero. Suggested
    /// default: `5.0` (→ effective `5.0`).
    ///
    /// Set to `0.0` to disable the relief.
    pub save_pressure_known_playable_relief: f64,
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
    /// - `+critical_exposure_delta_weight * threat` if the actor clue-touched the chop card.
    /// - `−critical_exposure_delta_weight * threat` if the card remains unsaved **and** the
    ///   actor had at least one clue token (a save was available but was skipped).
    ///
    /// `threat = 1 / (1 + 4 × buffer)` where `buffer` is the count of known-playable or
    /// known-trash cards in the chop owner's hand — identical to the formula used by the leaf
    /// evaluator's `critical_exposure_score`.  This keeps the immediate incentive proportional
    /// to actual discard risk: a critical 5 buffered behind three known-playable 1s yields
    /// `threat ≈ 0.077`, so the immediate bonus is ~13× smaller than for a truly naked chop.
    ///
    /// Set to 0 to disable.
    pub critical_exposure_delta_weight: f64,
    /// Flat tempo penalty charged on every action that does **not** advance the play stacks
    /// (clues, discards, misplays). Successful plays pay 0 here; their value is credited by
    /// `play_progress_bonus`.
    ///
    /// Closes the horizon-arbitrage hole: every redundant clue that pads the search line to
    /// defer a discard past the leaf now reduces the line's score by one `tempo_cost_weight`
    /// unit, so the engine cannot earn a free advantage by postponing inevitable bad events.
    ///
    /// Must be at least as large as the spurious gain from postponing a typical discard by
    /// one ply (≈ 1.0–3.0 at default weights). Suggested default: `1.5`.
    ///
    /// Set to 0 to disable.
    pub tempo_cost_weight: f64,
    /// Per-ply discount subtracted from a successful play's `play_progress_bonus` for every
    /// ply the play is deferred from the root (`elapsed = number_of_players × 2 − remaining_depth`).
    ///
    /// Closes the *deferred-play* horizon arbitrage: when two lines play the same cards and reach
    /// the same horizon-capped `game_score`, the line that defers its plays banks extra leaf
    /// `known_playable` credit for cards left un-cashed in hand, out-scoring the line that played
    /// them efficiently. Charging a small per-ply discount makes an earlier play strictly beat an
    /// identical deferred one, breaking the tie in favour of cashing progress now.
    ///
    /// Kept well below `play_progress_weight` so a productive play is always net-positive; only
    /// reduces the immediate bonus, so search `upper_bound` admissibility is preserved.
    ///
    /// Set to 0 to disable.
    pub play_deferral_weight: f64,
    /// Per-ply discount subtracted from the **`game_score`** a successful play realizes, scaled
    /// by `elapsed = number_of_players × 2 − remaining_depth` (continued past the leaf into the
    /// quiescence tail). Where [`Self::play_deferral_weight`] only discounts the small
    /// `play_progress_bonus`, this one bites on the dominant `score_weight` reward.
    ///
    /// **Only consumed by the quiescence tail** (`quiescence-search` feature). Beyond the horizon,
    /// a tail play resolves into `game_score` at full `score_weight`, so without this a line that
    /// *manufactures* extra tail-cashable plays (e.g. cluing more cards playable) out-scores a line
    /// that simply cashes a guaranteed play now — the deferral arbitrage relocated past the leaf.
    /// Discounting the realized score per deferred ply restores "cash now > set up for later".
    ///
    /// Kept well below `score_weight` so a deep tail play stays net-positive (it still advances the
    /// stacks); only lowers leaf values, so `upper_bound` admissibility is preserved.
    ///
    /// Set to 0 to disable.
    pub score_deferral_weight: f64,
    /// Multiplier for the **future-potential** leaf term: `future_potential_weight * clue_demand`,
    /// where `clue_demand = Σ_{untouched cards} max(P(playable), P(critical))` (see
    /// [`Self::clue_demand`]). This is the independent-leaf-term fix for the search-horizon
    /// "dry state" effect: it credits the *statistical* expected value of unclued cards in hands —
    /// including freshly drawn cards, which the search otherwise treats as having zero action value.
    ///
    /// The existing `clue_demand` was only ever consumed as a *multiplier* on the clue-token term
    /// (`clue_token_weight * harmonic(n) * (1 + clue_demand_weight * demand)`), so its credit
    /// collapsed to ~0 whenever the team had no tokens (`harmonic(0) = 0`) — exactly the dry,
    /// token-starved positions where the horizon artifact bites. Surfacing the same probability
    /// mass as a standalone term makes leaf valuations depth-stable: a line that cashes a play and
    /// draws a fresh card no longer scores worse purely because the new card is unknown.
    ///
    /// EV interpretation: `clue_demand` sums `P(playable)`-style mass per untouched card, so with
    /// `future_potential_weight ≈ known_playable_weight` an unknown card is valued at
    /// `P(playable) × (known-playable value)`, consistent with how a *known* playable scores 1.0.
    ///
    /// **Disabled by default (`0.0`), pending evidence.** The full suite passes at default depth
    /// with this active at `0.5` or `1.0` (no regressions), but a depth-increase experiment showed
    /// it does *not* resolve the residual horizon failures: across competing root lines the term is
    /// a near-constant offset (e.g. `+2.9` vs `+3.0`), so it cannot rebalance them. Those failures
    /// are driven by `game_score`-term dominance over long speculative play-cascades that deeper
    /// search uncovers — a distinct artifact this term was not designed to address. Kept here as a
    /// tunable, well-behaved leaf term: set `> 0` to re-enable.
    pub future_potential_weight: f64,
}

impl Default for DefaultEvaluator {
    fn default() -> Self {
        DefaultEvaluator {
            score_weight: 10.0_f64,
            strike_penalties: [0.0_f64, 10.0_f64, 30.0_f64, 1000.0_f64],
            pace_weight: 1.0_f64,
            efficiency_weight: 1.9_f64,
            critical_exposure_weight: 3.0_f64,
            lost_score_ceiling_weight: 30.0_f64,
            empathy_weight: 0.0_f64,
            good_touch_penalty: 25.0_f64,
            forced_save_bad_touch_relief: 15.0_f64,
            forced_stall_bad_touch_relief: 15.0_f64,
            clue_token_weight: 0.6_f64,
            clue_demand_weight: 0.05_f64,
            clue_precision_weight: 0.4_f64,
            known_playable_weight: 1.0_f64,
            team_empathy_weight: 0.3_f64,
            signal_ignored_penalty_weight: 5.0_f64,
            misinformation_weight: 0.0_f64,
            parked_dead_weight_penalty: 6.0_f64,
            play_progress_weight: 2.0_f64,
            potential_bad_touch_penalty: 9.0_f64,
            bottom_deck_risk_weight: 0.8_f64,
            discard_while_known_playable_penalty: 10.0_f64,
            save_pressure_known_playable_relief: 5.0_f64,
            critical_exposure_delta_weight: 0.5_f64,
            tempo_cost_weight: 2.5_f64,
            play_deferral_weight: 0.15_f64,
            score_deferral_weight: 0.75_f64,
            future_potential_weight: 0.0_f64,
        }
    }
}

impl DefaultEvaluator {
    /// Bitmask of card identities that are critical: exactly one copy remains outside the
    /// discard pile and the card is still needed (not yet on the play stacks).
    pub(crate) fn critical_mask(
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
    pub(crate) fn critical_exposure_score(
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

            // First pass: classify every card in chop order (chop first / oldest-to-newest).
            // `buffer` (known-playable + known-trash) is a *global* count: the holder can play
            // or trash-discard those instead of any critical regardless of position, so they
            // shield every critical in the hand. The non-buffer cards are recorded in order so
            // the second pass can measure each critical's *positional* protection.
            let mut buffer: u32 = 0;
            // Per non-buffer card, in chop order: (touched, truth-critical, truth-known).
            let mut run: smallvec::SmallVec<[(bool, bool, bool); 6]> = smallvec::SmallVec::new();
            for &deck_idx in hand.cards().iter().rev() {
                if pov.is_known_trash(deck_idx)
                    || Self::card_known_playable(&pov, deck_idx, playable_mask)
                {
                    buffer += 1;
                    continue;
                }
                let truth_id = truth.card_identity(deck_idx);
                let is_critical = truth_id.map_or(false, |id| (1u64 << id) & critical_mask != 0);
                run.push((pov.is_touched(deck_idx), is_critical, truth_id.is_some()));
            }

            // Second pass: walk that run, accumulating the protection in front of each
            // truth-critical card:
            //
            // - `touched_shield`: touched non-playable cards whose truth identity is non-
            //   critical. These are held indefinitely (never discarded) and sit in front of
            //   the critical, so saving such a card protects it at ×4 — and, just as
            //   importantly, the term must not *penalise* the save: a card going untouched →
            //   touched would otherwise just leave the chop-eligible run and bump the critical
            //   one rank closer to chop.
            // - `rank`: every untouched chop-eligible card seen so far (non-critical, unknown,
            //   or critical) — the holder's discard order. Each only pushes a later critical
            //   one position away (×1).
            //
            // Cards *behind* a critical (newer / further from chop) are reached only after it
            // and contribute nothing to its protection.
            let mut touched_shield: u32 = 0;
            let mut rank: u32 = 0;
            for &(touched, is_critical, truth_known) in &run {
                if touched {
                    if truth_known && !is_critical {
                        touched_shield += 1;
                    }
                    continue;
                }
                if is_critical {
                    let threat = 1.0 / (1.0 + rank as f64 + 4.0 * (buffer + touched_shield) as f64);
                    total += threat;
                }
                // Unknown-identity cards contribute no threat here (probabilistic risk is
                // BDR's job) but, like every chop-eligible card, push later criticals back.
                rank += 1;
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
    ///
    /// Thin wrapper over the engine's single "known playable" predicate,
    /// [`PlayerPOV::is_playable_with_playable_mask`], passing a precomputed
    /// `playable_mask` to avoid recomputing it per card in evaluator hot paths. The
    /// mask-less [`PlayerPOV::is_playable`] (and hence `is_hand_locked`) route through
    /// the same method, so all consumers share one definition.
    pub(crate) fn card_known_playable(
        pov: &dyn PlayerPOV,
        deck_idx: CardDeckIndex,
        playable_mask: VariantCardsBitField,
    ) -> bool {
        pov.is_playable_with_playable_mask(deck_idx, playable_mask)
    }

    /// Sum of identity-bits eliminated across all clued cards in every player's own hand.
    ///
    /// For each clued card, a player who knows exactly one identity (popcount=1) contributes
    /// `max_identities - 1` to the sum; a card with all identities open contributes 0.
    /// Higher is better — it means clues have conveyed more precise information.
    pub(crate) fn empathy_precision(
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
    pub(crate) fn known_playable_in_hands(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        truth: &dyn PlayerPOV,
    ) -> f64 {
        let num_players = static_data.number_of_players as usize;
        let playable_mask = table_state.playable_cards(static_data);
        let mut total = 0.0f64;
        // A playable identity can only be played once: the stack advances by exactly one when
        // the first copy plays, after which any remaining copies of that identity are dead. So a
        // playable identity the searcher can pin down (via truth view) is credited at most once,
        // globally across all hands. This prevents over-rewarding a clue that touches several
        // duplicate copies of the same playable (e.g. a rank-1 clue landing on three P1s), which
        // is a bad touch, not three plays.
        let mut credited_truth_ids: VariantCardsBitField = 0;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                // Detect the misinformation case: searcher knows the truth and it's
                // unplayable. Skip even if the holder's local knowledge thinks otherwise.
                let truth_id = truth.card_identity(idx as CardDeckIndex);
                if let Some(truth_id) = truth_id {
                    if (1u64 << truth_id) & playable_mask == 0 {
                        continue;
                    }
                }
                // Does the holder know (by any priority) that this card is playable?
                let counts = {
                    // Priority 1: convention signal
                    if pk.signals[idx]
                        .iter()
                        .any(|s| matches!(s, Signal::Play { .. }))
                    {
                        true
                    } else if let Some(inferred) = pk.inferred_identities[idx] {
                        // Priority 2: convention-inferred identity (currently unpopulated)
                        let bits = inferred.as_bits();
                        bits != 0 && (bits & playable_mask) == bits
                    } else {
                        // Priority 3: game-rule empathy from Deck (global)
                        let bits = table_state
                            .deck
                            .get_global_empathy(idx as CardDeckIndex)
                            .as_bits();
                        bits != 0 && (bits & playable_mask) == bits
                    }
                };
                if !counts {
                    continue;
                }
                // A playable identity the searcher can pin down via truth is credited at most
                // once, globally: only the first copy will play, after which the stack advances
                // and the rest are dead. This stops a bad-touch clue that lands on several
                // duplicate copies of one playable (e.g. a rank-1 clue touching three P1s) from
                // being scored as several independent plays.
                if let Some(truth_id) = truth_id {
                    let id_bit = 1u64 << truth_id;
                    if id_bit & credited_truth_ids != 0 {
                        continue;
                    }
                    credited_truth_ids |= id_bit;
                }
                total += 1.0;
            }
        }
        total
    }

    /// Standing count of clue-touched cards that are **dead weight**: their true identity is
    /// no longer still-needed (already played, or every remaining copy is a duplicate), yet
    /// the holder still believes them potentially useful (their empathy overlaps the
    /// still-needed set), so they will sit in hand rather than be discarded — and may even be
    /// misplayed into a strike.
    ///
    /// This is the leaf-resident counterpart to the one-time `good_touch_penalty` charged at
    /// clue time. A bad touch's true cost is not the instant of cluing — it is that the dead
    /// duplicates stay parked across the whole rollout, occupying slots and inviting a strike
    /// the short search horizon never reaches. Charging it at every leaf makes the search keep
    /// paying for the parked cards instead of recouping the one-time charge with optimistic
    /// plays elsewhere. Only the searcher-resolvable (truth-visible) cards are counted; an
    /// own-hand card the searcher cannot see contributes nothing.
    pub(crate) fn parked_dead_weight(
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        truth: &dyn PlayerPOV,
    ) -> f64 {
        let still_needed = still_needed_cards_mask(table_state, static_data);
        let touched = table_state.clue_touched_cards;
        let num_players = static_data.number_of_players as usize;
        let variant = &static_data.variant;
        let mut count = 0.0f64;
        for p in 0..num_players {
            let pk = team_knowledge.player(p);
            let mut hand = pk.own_hand & touched;
            while hand != 0 {
                let idx = hand.trailing_zeros() as usize;
                hand &= hand - 1;
                // Only count cards the searcher can pin to a truth identity.
                let Some(truth_id) = truth.card_identity(idx as CardDeckIndex) else {
                    continue;
                };
                // Truth identity still useful → not dead weight.
                if (1u64 << truth_id) & still_needed != 0 {
                    continue;
                }
                // Only a bad touch the holder *cannot recognise as trash* is a liability: the
                // card is believed useful, so it is parked in hand and risks a misplay-strike.
                // A touched-but-dead card whose empathy has collapsed to pure trash is harmless —
                // the holder will simply discard it (e.g. a rank-2 clue whose only still-useful 2
                // is locked onto the focus makes every other touched 2 provably trash). Charging
                // those would punish a perfectly good clue for an unavoidable, deducible bad touch.
                let empathy = pk
                    .combined_possible_identities(idx as CardDeckIndex, table_state, variant)
                    .as_bits();
                if empathy & still_needed == 0 {
                    continue;
                }
                count += 1.0;
            }
        }
        count
    }

    /// Fraction of identity uncertainty eliminated across all own-hand cards for all players.
    ///
    /// Each card contributes `(max_identities − popcount) / max_identities` ∈ [0, 1).
    /// A fully-unknown card contributes 0; a fully-resolved card contributes near 1.
    /// Uses game-rule empathy from Deck — convention signals and inferred identities are captured
    /// separately by `known_playable_in_hands` and `resolved_card_count`.
    pub(crate) fn team_empathy_score(
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
                    max_identities.saturating_sub(popcount.min(max_identities)) as f64 * inv_max;
            }
        }
        total
    }

    /// Harmonic sum H(n) = 1 + 1/2 + 1/3 + … + 1/n.
    ///
    /// Models the scarcity value of a clue bank with `n` whole tokens: the first token is worth
    /// 1.0, the second 0.5, the third 0.33, etc. Going from 0 → 1 is the largest jump; going
    /// from 7 → 8 adds only 0.125.
    pub(crate) fn harmonic(n: u8) -> f64 {
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
    pub(crate) fn clue_demand(table_state: &TableState, static_data: &StaticGameData) -> f64 {
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
    pub(crate) fn misinformation_score(
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
                    total += 1.0;
                } else {
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
    /// Uses a two-term expected-value model per eligible identity `id`:
    ///
    ///   `P(card_is_id)  ×  loss_if_lost(id)`
    ///
    /// - **Term A** `P(card_is_id)` = `1 / popcount` — uniform over empathy.
    /// - **Term C** `loss_if_lost` = achievable ranks from `rank_idx` upward before the
    ///   first fully-discarded rank (already-lost ranks are excluded from the loss).
    ///
    /// The model is **deck-position-independent**: turning a card critical is charged the same
    /// whether it happens early or late. We deliberately *do not* multiply by a Laplace
    /// `1 / deck_remaining` bottom-card probability — under that model the penalty for stranding a
    /// teammate into dumping a soon-to-be-critical card collapsed to a fraction of a point early in
    /// the game, when the team can least afford to be cavalier about it. The cost of converting a
    /// card from two live copies to one (or one to zero) is treated as a fixed liability, not a
    /// gamble on where the survivor sits in the deck.
    ///
    /// An identity is skipped when: it has only one total copy (terminal); its rank is
    /// the top of the stack; its suit is already past this rank; discarding doesn't drive
    /// remaining copies to exactly 1; or any other hand visibly holds the surviving copy.
    pub(crate) fn bottom_deck_risk_score(
        discarded_deck_idx: CardDeckIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
    ) -> f64 {
        let variant = &static_data.variant;
        let stacks_size = variant.stacks_size as usize;
        // When the evaluating POV doesn't know the card's identity, fall back to that POV's
        // own inferred empathy — not the deck's global empathy. The deck's global empathy is
        // truth-narrowed (every starting-hand card gets `reveal_card`'d by the scenario loader),
        // so consulting it here would leak the discarder's own-card identity into their own
        // risk assessment. The POV's `inferred_identities` correctly reflects only what the
        // evaluating player can deduce.
        let (empathy, popcount) = if let Some(known_id) = truth.card_identity(discarded_deck_idx) {
            (1u64 << known_id, 1.0f64)
        } else {
            let e = truth.inferred_identities(discarded_deck_idx).as_bits();
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
                continue;
            }
            let suit = id / stacks_size;
            if table_state.playing_stacks.stack_size(suit) as usize > rank_idx {
                continue;
            }
            let already_discarded =
                table_state.discard_pile.copies_of(id as VariantCardId) as usize;
            if total_copies.saturating_sub(already_discarded + 1) != 1 {
                continue;
            }
            if (visible_ids >> id) & 1 != 0 {
                continue;
            }
            let p_card_is_id = 1.0 / popcount;
            let mut loss_count = 0usize;
            for r in rank_idx..stacks_size {
                let cid = suit * stacks_size + r;
                let total_r = variant.card_copies_count_by_id[cid] as usize;
                if table_state.discard_pile.copies_of(cid as VariantCardId) as usize >= total_r {
                    break;
                }
                loss_count += 1;
            }
            score += p_card_is_id * loss_count as f64;
        }
        score
    }

    /// True iff `actor` currently holds at least one card the search's play
    /// generators would offer them — i.e. a card matched by either
    /// `BlindPlay` (via `Signal::Play`) or `PlayKnownPlayable` (via combined
    /// inferred ⊆ playable). Delegates to [`Self::card_known_playable`].
    pub(crate) fn actor_has_known_playable(
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

    /// Count of known-playable cards held by the player behind `pov`.
    ///
    /// Delegates to [`Self::card_known_playable`] for each card in the player's hand so the
    /// definition of "known playable" is consistent across all evaluator terms.
    pub(crate) fn count_known_playable(
        pov: &dyn PlayerPOV,
        playable_mask: VariantCardsBitField,
    ) -> u32 {
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

    /// True when the team is under *Save-Principle pressure* from `actor`'s seat: some **other**
    /// player's unclued chop holds a Save-Principle-protected card ([`is_save_principle_protected`])
    /// and that holder has no known-playable of their own — so, absent intervention, they are in
    /// concrete danger of discarding the protected card on their turn.
    ///
    /// `truth` resolves the teammates' chop identities (the searcher's POV sees every hand). Used
    /// to soften [`Self::discard_while_known_playable_penalty`]: when a protected card is about to
    /// be lost, a player spending the turn on the save instead of cashing a known play is excused.
    pub(crate) fn save_principle_card_in_danger(
        actor: PlayerIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        truth: &dyn PlayerPOV,
    ) -> bool {
        let num_players = static_data.number_of_players as usize;
        let playable_mask = table_state.playable_cards(static_data);
        for p in 0..num_players {
            if p == actor {
                continue;
            }
            let pk = team_knowledge.player(p);
            let pov = LightweightPlayerPOV::new(p, pk, team_knowledge, table_state, static_data);
            let Some(chop) = get_chop_index(p, &pov) else {
                continue;
            };
            // A holder with a known-playable plays it rather than discarding the chop, so the
            // protected card is not actually in danger this turn.
            if Self::count_known_playable(&pov, playable_mask) > 0 {
                continue;
            }
            let Some(chop_id) = truth.card_identity(chop) else {
                continue;
            };
            if is_save_principle_protected(chop_id, truth) {
                return true;
            }
        }
        false
    }
}
