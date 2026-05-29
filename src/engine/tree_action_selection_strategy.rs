use crate::engine::action_outcome::ActionOutcome;
use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::tech::clue_burn::ClueBurn;
use crate::engine::convention::hgroup::tech::five_stall::FiveStall;
use crate::engine::convention::hgroup::tech::low_level_stall::LowLevelStall;
use crate::engine::convention::hgroup::tech::tempo_clue::TempoClue;
use crate::engine::decision_tree::{LineStep, Score, ScoredNode};
use crate::engine::evaluator::{DefaultEvaluator, Evaluator, ScoreBreakdown};
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::engine::play_resolver::TruthPovResolver;
use crate::game::action::game_action::GameAction;
use crate::game::static_game_data::StaticGameData;
use rayon::prelude::*;
use smallvec::SmallVec;

/// Inline capacity for candidate action lists. 20 covers all realistic Hanabi scenarios
/// (5-card hands × play+discard + a handful of convention-proposed clues) without spilling
/// to the heap on the hot recursive path.
const CANDIDATE_INLINE_CAP: usize = 20;

/// Selects which subjective-action policy [`TreeActionSelectionStrategy::pick_subjective`]
/// uses at inner plies. Single-line toggle: change this constant to swap policies.
///
/// - `SubjectivePolicy::Recursive` — A-style: each inner player runs their own forward
///   sub-search from their own POV and returns the best-scoring candidate. Expressive
///   but can compound POV-restriction error at deep plies.
/// - `SubjectivePolicy::Priority` — H-Group-style: each inner player picks the
///   highest-priority tech proposal that fires for them. Deterministic, cheap, no
///   recursive POV compounding.
const SUBJECTIVE_POLICY: SubjectivePolicy = SubjectivePolicy::Recursive;

#[derive(Copy, Clone, PartialEq, Eq)]
enum SubjectivePolicy {
    Recursive,
    Priority,
}

/// Pre-allocated triangular PV table used to record the principal variation without
/// heap-allocating inside the search hot path.
///
/// Row `d` holds the PV for the call at remaining depth `d` (up to `d` steps).
/// All rows are pre-allocated to their maximum capacity at construction time, so
/// `push` and `extend` during search never trigger a reallocation.
struct PvTable {
    rows: Vec<Vec<LineStep>>,
}

impl PvTable {
    fn new(max_depth: usize) -> Self {
        let rows = (0..=max_depth).map(|d| Vec::with_capacity(d)).collect();
        PvTable { rows }
    }

    /// Write the PV for `depth` by prepending `step` to the child PV at `depth - 1`.
    ///
    /// Uses `split_at_mut` to hold simultaneous borrows of two rows.
    fn set_pv(&mut self, depth: usize, step: LineStep) {
        debug_assert!(depth > 0 && depth < self.rows.len());
        let (lower, upper) = self.rows.split_at_mut(depth);
        let child = &lower[depth - 1];
        let current = &mut upper[0];
        current.clear();
        current.push(step);
        for s in child {
            current.push(s.clone());
        }
    }

    fn pv_at(&self, depth: usize) -> &[LineStep] {
        &self.rows[depth]
    }
}

/// An action together with the tech that proposed it.
#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub action: GameAction,
    pub tech_name: &'static str,
    pub priority: u8,
}

pub struct TreeActionSelectionStrategy {
    pub evaluator: Box<dyn Evaluator>,
}

impl Default for TreeActionSelectionStrategy {
    fn default() -> Self {
        TreeActionSelectionStrategy {
            evaluator: Box::new(DefaultEvaluator::default()),
        }
    }
}

impl TreeActionSelectionStrategy {
    /// Returns proposed actions with tech provenance, for use by `scored_actions` and debugging.
    ///
    /// Returns a `SmallVec` with inline storage for up to `CANDIDATE_INLINE_CAP` entries so
    /// the typical case never touches the heap on the recursive hot path.
    pub fn candidate_actions_with_provenance(
        pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> SmallVec<[ProposedAction; CANDIDATE_INLINE_CAP]> {
        let has_clue_tokens = pov.table_state().clue_token_bank.whole_clue_tokens_count() > 0;
        let mut proposed: SmallVec<[ProposedAction; CANDIDATE_INLINE_CAP]> = convention_set
            .techs()
            .iter()
            .flat_map(|tech| {
                let tech_name = tech.name();
                let priority = tech.interpretation_priority();
                tech.game_actions(pov)
                    .into_iter()
                    .filter(move |a| has_clue_tokens || !matches!(a, GameAction::Clue { .. }))
                    .map(move |action| ProposedAction {
                        action,
                        tech_name,
                        priority,
                    })
            })
            .collect();

        if proposed.is_empty() {
            for (tech, name) in [
                (&FiveStall as &dyn ConventionTech, FiveStall.name()),
                (&TempoClue as &dyn ConventionTech, TempoClue.name()),
                (&ClueBurn as &dyn ConventionTech, ClueBurn.name()),
                (&LowLevelStall as &dyn ConventionTech, LowLevelStall.name()),
            ] {
                let actions: SmallVec<[ProposedAction; CANDIDATE_INLINE_CAP]> = tech
                    .game_actions(pov)
                    .into_iter()
                    .filter(|a| has_clue_tokens || !matches!(a, GameAction::Clue { .. }))
                    .map(|action| ProposedAction {
                        action,
                        tech_name: name,
                        priority: u8::MAX,
                    })
                    .collect();
                if !actions.is_empty() {
                    proposed = actions;
                    break;
                }
            }
        }

        // Dedup by action, keeping the first (highest-priority) tech that proposed it.
        // Linear scan is faster than HashSet for the small candidate counts seen in practice.
        let mut seen: SmallVec<[GameAction; CANDIDATE_INLINE_CAP]> = SmallVec::new();
        proposed.retain(|p| {
            if seen.contains(&p.action) {
                false
            } else {
                seen.push(p.action.clone());
                true
            }
        });

        // DiscardKnownTrash takes priority: if a known-trash discard was proposed, drop all
        // DiscardChop candidates so the search only considers the safe discard.
        if proposed.iter().any(|p| p.tech_name == "DiscardKnownTrash") {
            proposed.retain(|p| p.tech_name != "DiscardChop");
        }

        // Endgame pace constraint: when pace ≤ 0 the team cannot reach max score if any
        // turn is wasted. Drop all discards when a play is available — discarding is
        // mathematically forbidden at this point.
        let pace = pov.table_state().pace(pov.static_data());
        if pace <= 0
            && proposed
                .iter()
                .any(|p| matches!(&p.action, GameAction::Play { .. }))
        {
            proposed.retain(|p| !matches!(&p.action, GameAction::Discard { .. }));
        }

        // Move ordering: Play > Clue > Discard.
        proposed.sort_by_key(|p| match &p.action {
            GameAction::Play { .. } => 0,
            GameAction::Clue { .. } => 1,
            GameAction::Discard { .. } => 2,
            GameAction::Draw { .. } => 3,
        });
        proposed
    }

    fn leaf_breakdown(
        evaluator: &dyn Evaluator,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> ScoreBreakdown {
        evaluator.score_breakdown(state, truth, phantom_plays)
    }

    /// Per-action bonus applied along the search path.
    ///
    /// Sums every component of the evaluator's per-action terms (clue precision,
    /// signal penalty, play progress, team empathy, discard penalty, critical
    /// exposure delta, tempo cost). For clue actions, the positive part of
    /// `clue_precision_bonus` is then subtracted again so it cannot offset the
    /// action's own `tempo_cost`. This narrows a horizon-arbitrage hole where the
    /// search padded lines with productive clues because each clue's precision
    /// reward partly cancelled tempo, while a line that completed its work earlier
    /// accumulated `LowLevelStall` bombs paying full tempo with no offset.
    /// `team_empathy_delta_bonus` and `critical_exposure_delta_bonus` remain
    /// un-capped so they continue to rank between competing good clues.
    fn immediate_action_bonus(
        action: &GameAction,
        evaluator: &dyn Evaluator,
        state_before: &KnowledgeAwareGameState,
        state_after: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
        pre_phantom_plays: u8,
        post_phantom_plays: u8,
    ) -> Score {
        let actor = state_before.table_state().active_player_index;
        let signal_penalty = evaluator.signal_ignored_penalty(
            action,
            actor,
            static_data,
            &state_before.team_knowledge,
            state_before.table_state(),
        );
        let clue_bonus = if let GameAction::Clue {
            touched_card_deck_indexes,
            player_index,
            ..
        } = action
        {
            evaluator.clue_precision_bonus(
                touched_card_deck_indexes,
                *player_index,
                actor,
                truth,
                static_data,
                &state_after.team_knowledge,
                state_after.table_state(),
            )
        } else {
            0.0
        };
        let play_bonus = evaluator.play_progress_bonus(
            action,
            state_before,
            state_after,
            pre_phantom_plays,
            post_phantom_plays,
        );
        let team_empathy_bonus =
            evaluator.team_empathy_delta_bonus(action, state_before, state_after);
        let discard_penalty = evaluator.discard_action_penalty(action, actor, state_before, truth);
        let critical_exposure_delta = evaluator.critical_exposure_delta_bonus(
            action,
            actor,
            state_before,
            state_after,
            truth,
        );
        let tempo = evaluator.tempo_cost(
            action,
            state_before,
            state_after,
            pre_phantom_plays,
            post_phantom_plays,
        );
        // Horizon-arbitrage cap: the productive part of a clue's
        // `clue_precision_bonus` cannot offset its own `tempo_cost`. Without this,
        // the search padded lines with productive clues because each clue's net
        // precision reward partly cancelled tempo, while a line that completed its
        // work earlier accumulated `LowLevelStall` bombs paying full tempo with no
        // offset. Bad-touch clues (negative `clue_bonus`) keep their full penalty;
        // `team_empathy_delta_bonus` and `critical_exposure_delta_bonus` remain
        // un-capped so they still rank between competing good clues.
        let clue_offset = if matches!(action, GameAction::Clue { .. }) {
            -clue_bonus.max(0.0)
        } else {
            0.0
        };
        clue_bonus
            + signal_penalty
            + play_bonus
            + team_empathy_bonus
            + discard_penalty
            + critical_exposure_delta
            + tempo
            + clue_offset
    }

    /// Forward-simulate `depth` more plies from `state`, using `observer` as the
    /// truth POV for action-outcome resolution and leaf evaluation. No branching at
    /// inner plies — exactly one action is chosen per ply, picked by [`pick_subjective`]
    /// from the active player's POV.
    ///
    /// The principal variation is written into `pv_table` at row `depth`; callers read it via
    /// [`PvTable::pv_at`] after this function returns and prepend their own action.
    ///
    /// The returned score is the leaf evaluation **plus** the sum of per-action immediate bonuses
    /// (clue precision, signal/discard penalties, play progress, …) accumulated along the chosen
    /// line, so that good-touch violations and clue precision are visible at every ply, not just
    /// at the root.
    ///
    /// # Search model
    ///
    /// This is a **cooperative simulation** — there is no adversary, and only the root iterates
    /// over its own candidate actions. After the root picks `a`:
    /// 1. `a` is applied and the turn advances to the next player.
    /// 2. Each subsequent player is *simulated* picking one action via [`pick_subjective`]
    ///    (see [`SUBJECTIVE_POLICY`] for the policy variants). No branching over teammate
    ///    alternatives happens in this outer simulation — only one path forward per ply.
    /// 3. The leaf state is evaluated with the root observer's truth so the line value
    ///    accurately reflects what really happens (e.g. a bad-touch follow-on resolves
    ///    to a real strike rather than a phantom advance).
    /// 4. That score propagates back to rank `a` against the other root candidates.
    ///
    /// The decoupling between **how actions are picked** (each player from their own POV)
    /// and **how outcomes are resolved** (root truth throughout) keeps the search from
    /// "cheating" — an inner player can't refuse a play just because the root knows it
    /// will bomb — while still letting the root's analysis see real consequences.
    ///
    /// # `at_root_sim`
    ///
    /// `at_root_sim = true` selects **objective** resolution: the chosen action is
    /// applied with `observer` (the root truth POV) as the resolver. This is required
    /// at the root level so that bad-touch follow-ons resolve as real strikes — the
    /// root observer can see all teammates' hands and reports the actual card identity.
    ///
    /// `at_root_sim = false` selects **subjective** resolution: the chosen action is
    /// applied with the active player's own POV (`inner_pov`) as the resolver. This
    /// is required inside a recursive subjective sub-search ([`pick_subjective_recursive`])
    /// because there the `observer` is itself an inner player who can see the
    /// *acting* player's hand — using `observer` as truth would let plays of an
    /// actor's own hidden cards (e.g. `LowLevelStall` picking "play slot 1") falsely
    /// resolve as known successes, paying out stack advances the actor couldn't
    /// actually predict from her own POV.
    fn simulate_forward(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
        pv_table: &mut PvTable,
        observer: &dyn PlayerPOV,
        phantom_plays: u8,
        at_root_sim: bool,
    ) -> (Score, ScoreBreakdown) {
        if depth == 0 || state.table_state.is_terminal(static_data) {
            let breakdown = Self::leaf_breakdown(evaluator, state, observer, phantom_plays);
            tracing::trace!(
                target: "eel::search",
                depth = 0,
                terminal = state.table_state.is_terminal(static_data),
                leaf = %breakdown,
                "leaf_reached",
            );
            pv_table.rows[depth].clear();
            return (breakdown.total, breakdown);
        }

        let active = state.table_state.active_player_index;
        // Build the active player's POV from the *observer's* sight set so the inner
        // player can never use information the observer doesn't have. When the observer
        // is the root truth this is just the standard "cheating bot" guard; when the
        // observer is itself an inner POV (as inside [`pick_subjective`]) this also
        // prevents nested-leak chains.
        let observer_visible = observer.visible_cards();
        let active_pk = state.team_knowledge.player(active);
        let effective_visible = observer_visible & active_pk.visible_cards;
        let inner_pov = LightweightPlayerPOV::with_visible_cards(
            active,
            active_pk,
            &state.team_knowledge,
            &state.table_state,
            static_data,
            effective_visible,
        );

        // Policy: pick what the active player would do from *their* POV. See
        // [`SUBJECTIVE_POLICY`] for the variant in effect.
        let Some(chosen) = Self::pick_subjective(
            state,
            static_data,
            convention_set,
            evaluator,
            depth,
            &inner_pov,
        ) else {
            let breakdown = Self::leaf_breakdown(evaluator, state, observer, phantom_plays);
            pv_table.rows[depth].clear();
            return (breakdown.total, breakdown);
        };

        // Resolution policy depends on whether we're in the root simulation or in a
        // recursive subjective sub-search (see this function's `at_root_sim` doc-comment).
        // At the root, `observer` is the true root POV — using it lets bad-touch
        // follow-ons resolve as real strikes. Inside a recursive sub-search, `observer`
        // is the outer planning self (an inner player) who can see the active player's
        // hand; using it there would let plays of the active player's own hidden cards
        // falsely resolve as known successes. Under [`SubjectivePolicy::Priority`] the
        // recursive sub-search never fires, so `at_root_sim` is always `true`.
        let mut next = state.clone();
        let outcome: ActionOutcome = if at_root_sim {
            next.apply(
                &chosen.action,
                convention_set,
                &mut TruthPovResolver::new(observer),
            )
        } else {
            next.apply(
                &chosen.action,
                convention_set,
                &mut TruthPovResolver::new(&inner_pov),
            )
        };
        next.advance_turn();
        let child_phantom = phantom_plays + if outcome.is_phantom_play { 1 } else { 0 };

        // Score the per-ply contribution from the active player's POV so terms like
        // `discard_action_penalty`/BDR and `critical_exposure_delta_bonus` reflect what
        // the actor could actually see when they made the choice.
        let immediate = Self::immediate_action_bonus(
            &chosen.action,
            evaluator,
            state,
            &next,
            static_data,
            &inner_pov,
            phantom_plays,
            child_phantom,
        );

        let (subtree_score, leaf_bd) = Self::simulate_forward(
            &next,
            static_data,
            convention_set,
            evaluator,
            depth - 1,
            pv_table,
            observer,
            child_phantom,
            at_root_sim,
        );
        let total = subtree_score + immediate;
        tracing::trace!(
            target: "eel::search",
            depth,
            player = active,
            action = ?chosen.action,
            tech = chosen.tech_name,
            immediate,
            subtree_score,
            total,
            "ply_simulated",
        );
        pv_table.set_pv(
            depth,
            LineStep {
                action: chosen.action,
                tech_name: chosen.tech_name,
                immediate_bonus: immediate,
            },
        );
        (total, leaf_bd)
    }

    /// Dispatcher: selects between the recursive A-style and the priority-based
    /// subjective policies based on [`SUBJECTIVE_POLICY`]. The unused branch is
    /// dead-code-eliminated since `SUBJECTIVE_POLICY` is a compile-time constant.
    fn pick_subjective(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
        inner_pov: &dyn PlayerPOV,
    ) -> Option<ProposedAction> {
        match SUBJECTIVE_POLICY {
            SubjectivePolicy::Recursive => Self::pick_subjective_recursive(
                state,
                static_data,
                convention_set,
                evaluator,
                depth,
                inner_pov,
            ),
            SubjectivePolicy::Priority => Self::pick_subjective_priority(convention_set, inner_pov),
        }
    }

    /// A-style subjective policy: ask "what would the active player do?" by running their
    /// own forward-simulation evaluation from their POV across each candidate and picking
    /// the highest-scoring one.
    ///
    /// Resolution inside this sub-search uses `inner_pov` as truth — the active player
    /// resolves their own hidden cards as `KnownPlayableAmbiguous` (phantom plays), just
    /// as they would believe in reality. This is what lets a bad-touched card holder
    /// still *pick* the bad play (their POV says it's known-playable) while the outer
    /// [`simulate_forward`] applies that pick with the root's truth and sees the real
    /// strike.
    fn pick_subjective_recursive(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
        inner_pov: &dyn PlayerPOV,
    ) -> Option<ProposedAction> {
        let candidates = Self::candidate_actions_with_provenance(inner_pov, convention_set);
        let span = tracing::trace_span!(
            target: "eel::search",
            "pick_subjective_recursive",
            depth,
            player = inner_pov.active_player_index(),
            candidates = candidates.len(),
        );
        let _guard = span.enter();
        let mut best = f64::NEG_INFINITY;
        let mut best_proposed: Option<ProposedAction> = None;
        let node_ceiling = evaluator.upper_bound(state.table_state(), static_data, depth);
        for proposed in candidates {
            let mut next = state.clone();
            let outcome: ActionOutcome = next.apply(
                &proposed.action,
                convention_set,
                &mut TruthPovResolver::new(inner_pov),
            );
            next.advance_turn();
            let child_phantom = if outcome.is_phantom_play { 1 } else { 0 };
            let immediate = Self::immediate_action_bonus(
                &proposed.action,
                evaluator,
                state,
                &next,
                static_data,
                inner_pov,
                0,
                child_phantom,
            );
            let candidate_ceiling =
                evaluator.upper_bound(next.table_state(), static_data, depth - 1) + immediate;
            if candidate_ceiling <= best {
                tracing::trace!(
                    target: "eel::search",
                    action = ?proposed.action,
                    tech = proposed.tech_name,
                    candidate_ceiling,
                    best,
                    "subjective_candidate_pruned",
                );
                continue;
            }
            let mut pv_table = PvTable::new(depth.saturating_sub(1));
            let (subtree_score, _leaf_bd) = Self::simulate_forward(
                &next,
                static_data,
                convention_set,
                evaluator,
                depth - 1,
                &mut pv_table,
                inner_pov,
                child_phantom,
                false,
            );
            let score = subtree_score + immediate;
            tracing::trace!(
                target: "eel::search",
                action = ?proposed.action,
                tech = proposed.tech_name,
                immediate,
                subtree_score,
                score,
                "subjective_candidate_evaluated",
            );
            if score > best {
                best = score;
                best_proposed = Some(proposed);
                if best >= node_ceiling {
                    break;
                }
            }
        }
        best_proposed
    }

    /// Priority-based subjective policy: pick the highest-priority tech proposal
    /// that fires for the active player from their own POV.
    ///
    /// [`candidate_actions_with_provenance`] returns candidates already sorted by
    /// action type (Play > Clue > Discard) and, within each type, by the
    /// originating tech's `interpretation_priority` ascending — so the first entry
    /// is the action a player following H-Group convention priority would choose.
    /// No recursive sub-search: nested subjective simulation compounded POV-
    /// restriction error and
    /// added cost without buying meaningfully better picks.
    fn pick_subjective_priority(
        convention_set: &dyn ConventionSet,
        inner_pov: &dyn PlayerPOV,
    ) -> Option<ProposedAction> {
        let candidates = Self::candidate_actions_with_provenance(inner_pov, convention_set);
        let chosen = candidates.into_iter().next();
        if let Some(c) = &chosen {
            tracing::trace!(
                target: "eel::search",
                player = inner_pov.active_player_index(),
                action = ?c.action,
                tech = c.tech_name,
                priority = c.priority,
                "subjective_pick_priority",
            );
        }
        chosen
    }
}

impl ActionSelectionStrategy for TreeActionSelectionStrategy {
    fn select_active_player_action(
        &self,
        player_pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> GameAction {
        self.scored_actions(player_pov, convention_set)
            .into_iter()
            .next()
            .map(|n| n.action)
            .unwrap_or_else(|| {
                player_pov
                    .valid_actions()
                    .into_iter()
                    .next()
                    .expect("no valid actions available for the active player")
            })
    }
}

impl TreeActionSelectionStrategy {
    /// Score every root candidate and return the list, sorted best-first.
    /// Useful for debugging: lets callers inspect why the engine chose a particular action.
    pub fn scored_actions(
        &self,
        player_pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> Vec<ScoredNode> {
        let static_data = player_pov.static_data();
        let depth = (static_data.number_of_players * 2) as usize;
        let table_state = player_pov.table_state().clone();
        let next_deck_index =
            (crate::game::MAX_CARDS_IN_DECK as u8) - table_state.deck.current_size;
        let root_state = KnowledgeAwareGameState::from_parts(
            static_data.clone(),
            table_state,
            player_pov.team_knowledge().clone(),
            next_deck_index,
        );
        let candidates = Self::candidate_actions_with_provenance(player_pov, convention_set);
        let evaluator = self.evaluator.as_ref();
        let root_player = player_pov.active_player_index();
        let span = tracing::debug_span!(
            target: "eel::search",
            "scored_actions",
            player = player_pov.table_state().active_player_index,
            candidates = candidates.len(),
        );
        let subtree_depth = depth - 1;
        let mut nodes: Vec<ScoredNode> = candidates
            .into_vec()
            .into_par_iter()
            .map(|proposed| {
                let _guard = span.clone().entered();
                let root_knowledge = root_state.team_knowledge.player(root_player).clone();
                let truth_pov = LightweightPlayerPOV::new(
                    root_player,
                    &root_knowledge,
                    &root_state.team_knowledge,
                    root_state.table_state(),
                    static_data,
                );
                let mut next = root_state.clone();
                let outcome: ActionOutcome = next.apply(
                    &proposed.action,
                    convention_set,
                    &mut TruthPovResolver::new(&truth_pov),
                );
                next.advance_turn();
                let child_phantom = if outcome.is_phantom_play { 1 } else { 0 };
                let immediate_bonus = Self::immediate_action_bonus(
                    &proposed.action,
                    evaluator,
                    &root_state,
                    &next,
                    static_data,
                    &truth_pov,
                    0,
                    child_phantom,
                );
                let mut pv_table = PvTable::new(subtree_depth);
                let (leaf_score, leaf_breakdown) = Self::simulate_forward(
                    &next,
                    static_data,
                    convention_set,
                    evaluator,
                    subtree_depth,
                    &mut pv_table,
                    &truth_pov,
                    child_phantom,
                    true,
                );
                let total = leaf_score + immediate_bonus;
                let pv = pv_table.pv_at(subtree_depth);
                let mut line = Vec::with_capacity(pv.len() + 1);
                line.push(LineStep {
                    action: proposed.action.clone(),
                    tech_name: proposed.tech_name,
                    immediate_bonus,
                });
                line.extend_from_slice(pv);
                tracing::debug!(
                    target: "eel::search",
                    action = ?proposed.action,
                    tech = proposed.tech_name,
                    priority = proposed.priority,
                    leaf_score,
                    immediate_bonus,
                    total,
                    leaf = %leaf_breakdown,
                    line = ?line,
                    "candidate_scored",
                );
                ScoredNode::leaf(
                    proposed.action,
                    total,
                    proposed.tech_name,
                    line,
                    leaf_breakdown,
                )
            })
            .collect();
        nodes.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
        nodes
    }
}
