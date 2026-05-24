use crate::engine::decision_tree::Score;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::action::game_action::GameAction;
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
    /// `−tempo_cost_weight` per non-progressing action accumulated across the search line.
    /// Always 0.0 at a pure leaf evaluation — the value is folded in by `tempo_cost` as the
    /// search walks each ply.
    pub tempo_cost: f64,
    /// Sum of all terms above.
    pub total: f64,
}

impl std::fmt::Display for ScoreBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total={:.2} [score={:.1} -strike={:.1} +pace={:.1} -eff={:.1} -crit_exp={:.1} -ceil={:.1} +emp={:.1} +clue={:.1} +play={:.1} +team_emp={:.1} -misinfo={:.1} ±crit_delta={:.1} -tempo={:.1}]",
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
            self.tempo_cost,
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
            tempo_cost: 0.0,
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

    /// Immediate per-ply tempo cost.
    ///
    /// Returns a non-positive `Score` (penalty) for any action that consumes a turn
    /// without making forward progress on the play stacks. A `Play` that advances
    /// the score returns 0 here (its tempo value is already credited by
    /// `play_progress_bonus`).
    ///
    /// Fires on every action along the search line — clues, discards, and misplays
    /// all incur the cost; only successful plays escape it. This breaks the horizon
    /// arbitrage where the engine pads search lines with redundant clues to push
    /// discards past the leaf: every padding clue now reduces the line's score by
    /// `tempo_cost_weight`.
    fn tempo_cost(
        &self,
        _action: &GameAction,
        _pre: &KnowledgeAwareGameState,
        _post: &KnowledgeAwareGameState,
        _pre_phantom_plays: u8,
        _post_phantom_plays: u8,
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
pub(crate) const RECIPROCAL_LUT: [f64; 65] = build_reciprocal_lut();

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
pub(crate) const HARMONIC_LUT: [f64; 9] = build_harmonic_lut();
