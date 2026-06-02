use crate::engine::action_outcome::ActionOutcome;
use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::game_action_filter::GameActionFilter;
use crate::engine::convention::hgroup::h_group_core::is_good_touch_principle_compliant;
#[cfg(feature = "quiescence-search")]
use crate::engine::convention::hgroup::signal::Signal;
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
use crate::game::MAX_PLAYERS_IN_GAME;
use crate::game::action::game_action::GameAction;
use crate::game::static_game_data::StaticGameData;
// Parallel candidate scoring is disabled under `search-diagnostic` (sequential output is
// readable and attributable), so the rayon prelude is only needed in normal builds.
#[cfg(not(feature = "search-diagnostic"))]
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
        is_early_game: bool,
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

        // Early Game restriction: while still in the Early Game, a player may not discard their
        // chop if they have a productive, good-touch-compliant alternative tech available (a play,
        // a known-trash discard, or a GTP-compliant clue — never a stall fallback, which are not in
        // `proposed`). Expressed as a `GameActionFilter` so the rule lives in the convention layer.
        let has_productive_alternative = proposed.iter().any(|p| {
            p.tech_name != "DiscardChop"
                && match &p.action {
                    GameAction::Play { .. } | GameAction::Discard { .. } => true,
                    GameAction::Clue {
                        player_index,
                        touched_card_deck_indexes,
                        clue,
                        ..
                    } => is_good_touch_principle_compliant(
                        clue,
                        player_index,
                        touched_card_deck_indexes,
                        pov,
                    ),
                    GameAction::Draw { .. } => false,
                }
        });
        let early_game_filter =
            GameActionFilter::early_game_chop_discard(is_early_game, has_productive_alternative);
        proposed.retain(|p| early_game_filter.apply(&p.action, pov));

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
        // The planning POV used to resolve the *consequences* of a discard (BDR). Distinct from
        // `truth` (the acting player's POV): a player can't see their own discarded card, so scoring
        // the loss from the actor's POV diluted the real cost across their empathy. `observer` is the
        // same POV the leaf is evaluated from (root truth at the root sim, the inner planner inside a
        // recursive sub-search), so a discarded card the planner *can* see is costed at its true
        // identity. This never lets a player *choose* better with hidden info — picking still happens
        // from the actor's POV — it only makes the line's cost accounting honest. See
        // [`Self::simulate_forward`]'s `observer` doc-comment.
        observer: &dyn PlayerPOV,
        pre_phantom_plays: u8,
        post_phantom_plays: u8,
        remaining_depth: usize,
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
            clue,
            ..
        } = action
        {
            evaluator.clue_precision_bonus(
                clue,
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
        let mut play_bonus = evaluator.play_progress_bonus(
            action,
            state_before,
            state_after,
            pre_phantom_plays,
            post_phantom_plays,
        );
        // Deferred-play discount: a successful play loses a small amount per ply it is pushed
        // back from the root, so an earlier play strictly beats an identical deferred one. This
        // closes the horizon-arbitrage hole where a line that postpones its plays banks extra
        // leaf `known_playable` credit while reaching the same (horizon-capped) game score.
        if play_bonus > 0.0 {
            let root_depth = (static_data.number_of_players as usize) * 2;
            let elapsed = root_depth.saturating_sub(remaining_depth);
            play_bonus -= evaluator.play_deferral_penalty(elapsed);
        }
        let team_empathy_bonus =
            evaluator.team_empathy_delta_bonus(action, state_before, state_after);
        let discard_penalty =
            evaluator.discard_action_penalty(action, actor, state_before, observer);
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

    /// Stall-quiescence predicate (feature on): is the subjectively-chosen action a forced
    /// last-resort [`LowLevelStall`]? That tech is only reached via the fallback path in
    /// [`Self::candidate_actions_with_provenance`] once every productive tech (and the
    /// FiveStall/TempoClue/ClueBurn fallbacks) produced nothing — so its presence is a precise
    /// "out of real moves" signal that [`simulate_forward`] uses to settle the line early.
    #[cfg(feature = "stall-quiescence")]
    #[inline]
    fn is_forced_stall_leaf(tech_name: &str) -> bool {
        tech_name == LowLevelStall.name()
    }

    /// Stall-quiescence stub (feature off): never settles early, so [`simulate_forward`] applies
    /// the stall and recurses exactly as in the baseline — byte-for-byte unchanged.
    #[cfg(not(feature = "stall-quiescence"))]
    #[inline]
    fn is_forced_stall_leaf(_tech_name: &str) -> bool {
        false
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
    #[allow(clippy::too_many_arguments)]
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
        declined: [u8; MAX_PLAYERS_IN_GAME],
        // Cards playable in the *root* state, used to gate declined-play debt: only a play that
        // pre-existed at the root counts as "declined". A play the line itself unlocked (e.g. a
        // finesse connector cashed earlier in this very line) is not a lazy decline.
        root_playable: u64,
    ) -> (Score, ScoreBreakdown) {
        if depth == 0 || state.table_state.is_terminal(static_data) {
            // Quiescence extension (experimental, off by default): when this leaf still has a
            // pending *signalled* play, roll it forward so the play resolves into real
            // game_score instead of a static estimate. A no-op (returns `None`) when the
            // feature is disabled, when the leaf is already quiet, or below the root sim.
            if let Some(settled) = Self::quiescence_extend(
                state,
                static_data,
                convention_set,
                evaluator,
                observer,
                phantom_plays,
                at_root_sim,
                depth,
                declined,
                root_playable,
            ) {
                pv_table.rows[depth].clear();
                return settled;
            }
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
            at_root_sim,
        ) else {
            let breakdown = Self::leaf_breakdown(evaluator, state, observer, phantom_plays);
            pv_table.rows[depth].clear();
            return (breakdown.total, breakdown);
        };

        // Stall-quiescence (experimental, off by default): a `LowLevelStall` only fires when no
        // productive tech proposed a move, so it marks the search hitting the "no real move"
        // wall — an artifact of frozen information, since real draws would supply fresh actions.
        // Rather than simulate the stall and charge its consequences (wasted clue, trashed
        // critical, or a Priority-3 "play slot 1" strike) into this line's score, settle here on
        // the pre-stall state's static leaf. Banked signalled/known playables still count via the
        // leaf `known_playable` term, so downstream potential is valued, just not realized.
        if Self::is_forced_stall_leaf(chosen.tech_name) {
            let breakdown = Self::leaf_breakdown(evaluator, state, observer, phantom_plays);
            pv_table.rows[depth].clear();
            return (breakdown.total, breakdown);
        }

        // Resolution policy depends on whether we're in the root simulation or in a
        // recursive subjective sub-search (see this function's `at_root_sim` doc-comment).
        // At the root, `observer` is the true root POV — resolving against it lets bad-touch
        // follow-ons resolve as real strikes. Inside a recursive sub-search the *acting*
        // player's POV drives the decision, but resolution additionally consults the planning
        // `observer` (an inner player) for what it can already see — so a teammate's play or
        // discard resolves to its true consequence rather than an optimistic phantom. This is
        // safe because the observer can never see its own holder's hand, nor (via the
        // effective-visibility bound) the root player's hand, so neither the actor's
        // decision-time POV nor the root decision can use information it shouldn't. Under
        // [`SubjectivePolicy::Priority`] the recursive sub-search never fires, so
        // `at_root_sim` is always `true`.
        let mut next = state.clone();
        let _outcome: ActionOutcome = if at_root_sim {
            next.apply(
                &chosen.action,
                convention_set,
                &mut TruthPovResolver::new(observer),
            )
        } else {
            // Subjective sub-search: the *acting* player's POV drives the decision (candidate
            // generation above), but resolution also consults the planning `observer` so the
            // planner sees the real consequences of what it can already observe — a teammate's
            // discarded critical registering its lost ceiling, or a teammate's play resolving
            // to its true success/strike instead of an always-optimistic phantom. The observer
            // can never see its own holder's hand or (via the effective-visibility bound) the
            // root player's hand, so the actor's own cards stay hidden at decision time and no
            // root-hand information leaks back. See [`TruthPovResolver::with_observer`].
            next.apply(
                &chosen.action,
                convention_set,
                &mut TruthPovResolver::with_observer(&inner_pov, observer),
            )
        };
        next.advance_turn();
        // The deduped phantom count lives in the rollout state: it rises by one for a fresh
        // phantom claim and falls again when a later real play realizes that identity.
        let child_phantom = next.phantom_play_count();

        // Score the per-ply contribution from the active player's POV so terms like clue precision
        // and `critical_exposure_delta_bonus` reflect what the actor could actually see when they
        // made the choice. The discard consequence (BDR), however, is scored from `observer` — the
        // same planning POV the leaf uses — so losing a card the planner can identify is costed at
        // its true value rather than diluted across the discarder's own-card empathy.
        let immediate = Self::immediate_action_bonus(
            &chosen.action,
            evaluator,
            state,
            &next,
            static_data,
            &inner_pov,
            observer,
            phantom_plays,
            child_phantom,
            depth,
        );

        // Declined-play accounting (root truth simulation only — the tail that consumes this
        // runs only there). If the active player took a non-`Play` action while holding a
        // guaranteed play, record the debt so the quiescence tail cannot later re-credit that
        // deferred play for free. Closes the deferral arbitrage at its source rather than via a
        // blunt across-the-board discount.
        let child_declined = {
            #[cfg(feature = "quiescence-search")]
            {
                let mut d = declined;
                if at_root_sim
                    && !matches!(chosen.action, GameAction::Play { .. })
                    && Self::player_has_uncashed_signalled_play(
                        state,
                        static_data,
                        observer,
                        active,
                    )
                {
                    d[active] = d[active].saturating_add(1);
                }
                d
            }
            #[cfg(not(feature = "quiescence-search"))]
            {
                declined
            }
        };

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
            child_declined,
            root_playable,
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

    /// Quiescence stub when the feature is off: never extends, so [`simulate_forward`]
    /// falls through to the static leaf — byte-for-byte the baseline behavior.
    #[cfg(not(feature = "quiescence-search"))]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn quiescence_extend(
        _state: &KnowledgeAwareGameState,
        _static_data: &StaticGameData,
        _convention_set: &dyn ConventionSet,
        _evaluator: &dyn Evaluator,
        _observer: &dyn PlayerPOV,
        _phantom_plays: u8,
        _at_root_sim: bool,
        _depth: usize,
        _declined: [u8; MAX_PLAYERS_IN_GAME],
        _root_playable: u64,
    ) -> Option<(Score, ScoreBreakdown)> {
        None
    }

    /// Quiescence entry point (feature on): extend only a *loud* root-PV leaf — one with a
    /// pending signalled play the rollout can actually cash. Returns `None` (fall through to
    /// the static leaf) when below the root sim, not at the horizon, terminal, or already
    /// quiet. See [`Self::quiescence_rollout`] for the tail and [`Self::has_uncashed_signalled_play`]
    /// for why the predicate is signal-based rather than the god's-eye `known_playable` term.
    #[cfg(feature = "quiescence-search")]
    #[allow(clippy::too_many_arguments)]
    fn quiescence_extend(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        observer: &dyn PlayerPOV,
        phantom_plays: u8,
        at_root_sim: bool,
        depth: usize,
        declined: [u8; MAX_PLAYERS_IN_GAME],
        root_playable: u64,
    ) -> Option<(Score, ScoreBreakdown)> {
        if !(at_root_sim
            && depth == 0
            && !state.table_state.is_terminal(static_data)
            && Self::has_uncashed_signalled_play(state, static_data, observer))
        {
            return None;
        }
        let budget = (static_data.number_of_players * 2) as usize;
        Some(Self::quiescence_rollout(
            state,
            static_data,
            convention_set,
            evaluator,
            observer,
            phantom_plays,
            at_root_sim,
            budget,
            declined,
            root_playable,
        ))
    }

    /// Loud/quiet predicate for quiescence: does any hand hold a card whose holder *signally*
    /// knows to play it (a `Signal::Play`, or a convention-inferred all-playable identity)?
    ///
    /// This is deliberately tighter than the leaf `known_playable` term, which also counts
    /// cards playable only under the searcher's god's-eye global empathy (priority 3 in
    /// [`DefaultEvaluator::known_playable_in_hands`]). The quiescence rollout can only *cash*
    /// what the subjective holder will actually play (`PlayKnownPlayable` fires on signalled /
    /// inferred-playable cards) — so loudness must use the same notion of "playable" the
    /// rollout uses, or the tail spins discarding global-empathy "plays" it can never realize.
    #[cfg(feature = "quiescence-search")]
    fn has_uncashed_signalled_play(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
    ) -> bool {
        (0..static_data.number_of_players as usize)
            .any(|p| Self::player_has_uncashed_signalled_play(state, static_data, truth, p))
    }

    /// Per-player version of [`Self::has_uncashed_signalled_play`]: does `player` hold a card
    /// they *signally* know to play (a `Signal::Play`, or a convention-inferred all-playable
    /// identity that the searcher's `truth` confirms is genuinely playable)?
    ///
    /// Used both to compute team-wide loudness above and to detect a *declined play* — a player
    /// taking a non-`Play` action while this returns true is deferring a guaranteed play, which
    /// the tail must not later re-credit for free (see [`Self::quiescence_rollout`]).
    #[cfg(feature = "quiescence-search")]
    fn player_has_uncashed_signalled_play(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
        player: usize,
    ) -> bool {
        let playable_mask = state.table_state.playable_cards(static_data);
        let pk = state.team_knowledge.player(player);
        let mut hand = pk.own_hand;
        while hand != 0 {
            let idx = hand.trailing_zeros() as usize;
            hand &= hand - 1;
            // Skip cards the searcher knows are not actually playable (misinformation).
            if let Some(truth_id) = truth.card_identity(idx as u8) {
                if (1u64 << truth_id) & playable_mask == 0 {
                    continue;
                }
            }
            // Priority 1: convention play signal.
            if pk.signals[idx]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }))
            {
                return true;
            }
            // Priority 2: convention-inferred all-playable identity.
            if let Some(inferred) = pk.inferred_identities[idx] {
                let bits = inferred.as_bits();
                if bits != 0 && (bits & playable_mask) == bits {
                    return true;
                }
            }
        }
        false
    }

    /// Does the play in `action` cash a card the acting player believes is a **root-playable**
    /// identity (a `Signal::Play` whose `committed_identity` was playable at the root, or a
    /// convention-inferred identity set entirely within `root_playable`)?
    ///
    /// The declined-play debt is a per-player counter, not card-specific, so a player who declined a
    /// pre-existing play (debt recorded) and *also* holds a line-unlocked play (e.g. a finesse
    /// payoff) could have the debt wrongly consumed by the line-unlocked play's tail cash. Gating
    /// the repay on the *cashed* card being root-playable keeps the neutralization aimed at the
    /// pre-existing play it was meant for, and lets a genuine finesse payoff bank its score.
    ///
    /// Uses the acting player's own knowledge rather than `truth` so it works for the root player's
    /// own hand too (where `truth` cannot see the identity).
    #[cfg(feature = "quiescence-search")]
    fn cashed_play_is_root_playable(
        pk: &crate::engine::knowledge::player_knowledge::PlayerKnowledge,
        action: &GameAction,
        root_playable: u64,
    ) -> bool {
        let GameAction::Play {
            card_deck_index, ..
        } = action
        else {
            return false;
        };
        let di = *card_deck_index as usize;
        if let Some(committed_identity) = pk.signals[di].iter().find_map(|s| match s {
            Signal::Play {
                committed_identity, ..
            } => Some(*committed_identity),
            _ => None,
        }) {
            return (1u64 << committed_identity) & root_playable != 0;
        }
        if let Some(inferred) = pk.inferred_identities[di] {
            let bits = inferred.as_bits();
            return bits != 0 && (bits & !root_playable) == 0;
        }
        false
    }

    /// Quiescence tail: extend a *loud* leaf (one with an uncashed signalled play) until it
    /// goes quiet, so the returned `game_score` reflects pending plays actually resolving
    /// rather than a static estimate. Mirrors [`simulate_forward`]'s pick→apply→advance
    /// loop but (a) iterates instead of recursing, (b) credits only realized
    /// `play_progress_bonus` — intervening stalls/discards contribute nothing, so the
    /// extension can never manufacture a tempo/stall penalty — and (c) records no PV.
    ///
    /// Runs only under the root truth simulation (`at_root_sim == true`); inner subjective
    /// sub-searches keep the plain static leaf, which bounds cost and prevents nesting a
    /// quiescence rollout inside every inner pick.
    #[cfg(feature = "quiescence-search")]
    #[allow(clippy::too_many_arguments)]
    fn quiescence_rollout(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        observer: &dyn PlayerPOV,
        phantom_plays: u8,
        at_root_sim: bool,
        budget: usize,
        declined: [u8; MAX_PLAYERS_IN_GAME],
        root_playable: u64,
    ) -> (Score, ScoreBreakdown) {
        let mut cur = state.clone();
        let mut phantom = phantom_plays;
        let mut acc_play_bonus: Score = 0.0;
        let mut remaining = budget;
        // Per-player declined-play debt carried in from the in-horizon line (see
        // [`Self::simulate_forward`]). A tail play by a player with outstanding debt merely
        // repays a play they declined earlier, so its realized score is neutralized rather than
        // banked — closing the deferral arbitrage at its source.
        let mut declined = declined;
        // The tail begins one ply past the horizon, so its plays are the most-deferred of
        // all. `elapsed` starts at the leaf depth (`root_depth`) and advances per tail ply,
        // mirroring `immediate_action_bonus`'s `elapsed = root_depth - remaining_depth`.
        let root_depth = (static_data.number_of_players as usize) * 2;
        let mut elapsed = root_depth;

        loop {
            // Settle once quiet (no signalled play to cash), out of budget, or terminal.
            if remaining == 0
                || cur.table_state.is_terminal(static_data)
                || !Self::has_uncashed_signalled_play(&cur, static_data, observer)
            {
                let breakdown = Self::leaf_breakdown(evaluator, &cur, observer, phantom);
                return (breakdown.total + acc_play_bonus, breakdown);
            }

            let active = cur.table_state.active_player_index;
            let observer_visible = observer.visible_cards();
            let active_pk = cur.team_knowledge.player(active);
            let effective_visible = observer_visible & active_pk.visible_cards;
            let inner_pov = LightweightPlayerPOV::with_visible_cards(
                active,
                active_pk,
                &cur.team_knowledge,
                &cur.table_state,
                static_data,
                effective_visible,
            );

            // One ply of subjective lookahead is enough for a holder to cash a known play.
            // `pick_subjective` with `depth = 1` recurses into `simulate_forward` at depth 0
            // with `at_root_sim = false`, so the inner pick does not re-enter quiescence.
            let Some(chosen) = Self::pick_subjective(
                &cur,
                static_data,
                convention_set,
                evaluator,
                1,
                &inner_pov,
                at_root_sim,
            ) else {
                let breakdown = Self::leaf_breakdown(evaluator, &cur, observer, phantom);
                return (breakdown.total + acc_play_bonus, breakdown);
            };

            let mut next = cur.clone();
            let _outcome: ActionOutcome = next.apply(
                &chosen.action,
                convention_set,
                &mut TruthPovResolver::new(observer),
            );
            next.advance_turn();
            let child_phantom = next.phantom_play_count();

            // Credit only realized play progress; stalls/discards contribute nothing. Apply the
            // same deferred-play discount the in-horizon plies pay (`immediate_action_bonus`),
            // continued past the leaf so a tail play is strictly worse than the identical play
            // made earlier. Without this the tail is a penalty-free zone that re-opens the
            // horizon-arbitrage the discount exists to close — just relocated past the leaf.
            let mut play_bonus =
                evaluator.play_progress_bonus(&chosen.action, &cur, &next, phantom, child_phantom);
            if play_bonus > 0.0 {
                if declined[active] > 0
                    && Self::cashed_play_is_root_playable(active_pk, &chosen.action, root_playable)
                {
                    // This tail play repays a *pre-existing* play `active` *declined* earlier in
                    // this line (discarded/clued while holding a guaranteed play). The play still
                    // advances the stack — so its `score_weight` lands in `breakdown.total` below —
                    // but we subtract that full realized score here so re-cashing the deferred play
                    // banks none of it: cashing a play *now* strictly beats declining it and
                    // re-cashing it past the horizon. It then pays the same deferral discount as any
                    // other tail play (no exemption for the wasted turn), so the repaid play lands
                    // net *negative* — closing the arbitrage at its source, where the flat
                    // `score_deferral` discount alone only narrowed it.
                    //
                    // The repay is gated on the *cashed* card being root-playable
                    // ([`Self::cashed_play_is_root_playable`]): the debt is a per-player counter, so
                    // without the gate a player who declined a pre-existing play but also holds a
                    // line-unlocked play (a finesse payoff) would have the debt wrongly consumed by
                    // — and the payoff wiped from — that unrelated, genuinely new play.
                    declined[active] -= 1;
                    play_bonus = -evaluator.play_score_weight()
                        - evaluator.play_deferral_penalty(elapsed)
                        - evaluator.score_deferral_penalty(elapsed);
                } else {
                    play_bonus -= evaluator.play_deferral_penalty(elapsed);
                    // Also discount the realized `game_score` (folded into `breakdown.total` below)
                    // by how deep in the tail the play lands. Without this, a tail play banks full
                    // `score_weight`, so a line that defers/manufactures plays into the tail out-scores
                    // one that cashes a guaranteed play now — the horizon arbitrage relocated past the
                    // leaf. See `DefaultEvaluator::score_deferral_weight`.
                    play_bonus -= evaluator.score_deferral_penalty(elapsed);
                }
            } else if !matches!(chosen.action, GameAction::Play { .. })
                && Self::player_has_uncashed_signalled_play(&cur, static_data, observer, active)
            {
                // A non-play action inside the tail by a player who *could* have cashed a known
                // play is itself a decline; record the debt so their later tail play is neutralized.
                declined[active] = declined[active].saturating_add(1);
            }
            acc_play_bonus += play_bonus;

            cur = next;
            phantom = child_phantom;
            remaining -= 1;
            elapsed += 1;
        }
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
        outer: bool,
    ) -> Option<ProposedAction> {
        match SUBJECTIVE_POLICY {
            SubjectivePolicy::Recursive => Self::pick_subjective_recursive(
                state,
                static_data,
                convention_set,
                evaluator,
                depth,
                inner_pov,
                outer,
            ),
            SubjectivePolicy::Priority => {
                Self::pick_subjective_priority(convention_set, inner_pov, state.is_early_game())
            }
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
        outer: bool,
    ) -> Option<ProposedAction> {
        let candidates = Self::candidate_actions_with_provenance(
            inner_pov,
            convention_set,
            state.is_early_game(),
        );
        #[cfg(feature = "search-diagnostic")]
        {
            println!(
                "subjective_candidates_list (player={}, depth={}, outer={}):",
                inner_pov.active_player_index(),
                depth,
                outer
            );
            for c in &candidates {
                println!(
                    "  action={:?} tech={} priority={}",
                    c.action, c.tech_name, c.priority
                );
            }
        }
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
            // This nested sub-search scores from a fresh `phantom_plays = 0` base, so isolate its
            // phantom-claim ledger from the outer line's claims (whose absolute count would only
            // add a constant offset, but whose claimed identities could otherwise suppress a
            // legitimate inner phantom). Dedup still applies within this inner line.
            next.reset_phantom_claims();
            let _outcome: ActionOutcome = next.apply(
                &proposed.action,
                convention_set,
                &mut TruthPovResolver::new(inner_pov),
            );
            next.advance_turn();
            let child_phantom = next.phantom_play_count();
            let immediate = Self::immediate_action_bonus(
                &proposed.action,
                evaluator,
                state,
                &next,
                static_data,
                inner_pov,
                // Inside the inner subjective sub-search the planner *is* the acting player, so the
                // discard consequence is scored from their own POV too (no more-informed observer
                // exists here).
                inner_pov,
                0,
                child_phantom,
                depth,
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
            let (subtree_score, leaf_bd) = Self::simulate_forward(
                &next,
                static_data,
                convention_set,
                evaluator,
                depth - 1,
                &mut pv_table,
                inner_pov,
                child_phantom,
                false,
                // Inner subjective sub-searches never run the quiescence tail (`at_root_sim ==
                // false`), so declined-play debt is irrelevant here — start clean. `root_playable`
                // is likewise unused below the root sim; pass 0.
                [0; MAX_PLAYERS_IN_GAME],
                0,
            );
            let score = subtree_score + immediate;
            #[cfg(feature = "search-diagnostic")]
            {
                let pv = pv_table.pv_at(depth.saturating_sub(1));
                let mut line = Vec::with_capacity(pv.len() + 1);
                line.push(LineStep {
                    action: proposed.action.clone(),
                    tech_name: proposed.tech_name,
                    immediate_bonus: immediate,
                });
                line.extend_from_slice(pv);
                println!(
                    "subjective_candidate_evaluated (player={}, depth={}, outer={}): action={:?} tech={} immediate={} subtree_score={} score={} leaf={} line={:?}",
                    inner_pov.active_player_index(),
                    depth,
                    outer,
                    proposed.action,
                    proposed.tech_name,
                    immediate,
                    subtree_score,
                    score,
                    leaf_bd,
                    line
                );
            }
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
        is_early_game: bool,
    ) -> Option<ProposedAction> {
        let candidates =
            Self::candidate_actions_with_provenance(inner_pov, convention_set, is_early_game);
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
    /// Diagnostic helper: replay `actions` from `pov` and return the leaf
    /// [`ScoreBreakdown`] of the resulting state, plus the number of phantom plays
    /// accumulated along the way.
    ///
    /// Each action is resolved with the *acting* player's own POV as truth — exactly as the
    /// inner subjective sub-search does — so an ambiguous-but-playable card resolves as a
    /// phantom play rather than committing a specific stack. This lets a diagnostic compare
    /// what the search *internally* credits a line against where that same line actually lands:
    ///
    /// - Pass a **single-action** slice to get the depth-0 child leaf — one root action, no
    ///   rollout — isolating the leaf evaluator's verdict from any continuation.
    /// - Pass a full **principal variation** to see the leaf a specific line truly reaches.
    ///
    /// Encapsulates the clone-then-borrow dance otherwise required because [`apply`] takes
    /// `&mut state` while the truth POV borrows the same fields.
    ///
    /// [`apply`]: KnowledgeAwareGameState::apply
    pub fn replay_leaf_breakdown(
        &self,
        pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
        actions: &[GameAction],
    ) -> (ScoreBreakdown, u8) {
        let static_data = pov.static_data();
        let observer = pov.active_player_index();
        let table_state = pov.table_state().clone();
        let next_deck_index =
            (crate::game::MAX_CARDS_IN_DECK as u8) - table_state.deck.current_size;
        let mut state = KnowledgeAwareGameState::from_parts(
            static_data.clone(),
            table_state,
            pov.team_knowledge().clone(),
            next_deck_index,
        );
        for action in actions {
            let actor = state.table_state().active_player_index;
            // Clone the fields the truth POV borrows so `apply` can take `&mut state`.
            let actor_knowledge = state.team_knowledge.player(actor).clone();
            let team = state.team_knowledge.clone();
            let table = state.table_state().clone();
            let truth =
                LightweightPlayerPOV::new(actor, &actor_knowledge, &team, &table, static_data);
            let _ = state.apply(action, convention_set, &mut TruthPovResolver::new(&truth));
            state.advance_turn();
        }
        // Mirror the search's deduped accounting: the count is the popcount of the per-line
        // phantom-claim ledger, which already discounts duplicate-identity phantoms and releases
        // claims realized by a later real play.
        let phantom_plays = state.phantom_play_count();
        let observer_knowledge = state.team_knowledge.player(observer).clone();
        let team = state.team_knowledge.clone();
        let table = state.table_state().clone();
        let truth =
            LightweightPlayerPOV::new(observer, &observer_knowledge, &team, &table, static_data);
        let breakdown = self
            .evaluator
            .score_breakdown(&state, &truth, phantom_plays);
        (breakdown, phantom_plays)
    }

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
        let candidates = Self::candidate_actions_with_provenance(
            player_pov,
            convention_set,
            root_state.is_early_game(),
        );
        let evaluator = self.evaluator.as_ref();
        let root_player = player_pov.active_player_index();
        let span = tracing::debug_span!(
            target: "eel::search",
            "scored_actions",
            player = player_pov.table_state().active_player_index,
            candidates = candidates.len(),
        );
        let subtree_depth = depth - 1;
        // Cards playable in the root state, threaded through the rollout to gate declined-play
        // debt: only a play that pre-existed here counts as "declined" (see
        // [`Self::player_declined_root_playable`]).
        let root_playable = root_state.table_state().playable_cards(static_data);
        let eval_candidate = |proposed: ProposedAction| -> ScoredNode {
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
            let _outcome: ActionOutcome = next.apply(
                &proposed.action,
                convention_set,
                &mut TruthPovResolver::new(&truth_pov),
            );
            next.advance_turn();
            let child_phantom = next.phantom_play_count();
            let immediate_bonus = Self::immediate_action_bonus(
                &proposed.action,
                evaluator,
                &root_state,
                &next,
                static_data,
                &truth_pov,
                // At the root the acting player is the observer (root truth); a discard of the root
                // player's own card stays empathy-scored since not even root truth sees it.
                &truth_pov,
                0,
                child_phantom,
                depth,
            );
            // Seed the declined-play debt from the root action itself: if the root player chose a
            // non-`Play` while holding a guaranteed play, the quiescence tail must not later
            // re-credit that deferred play for free (see [`Self::quiescence_rollout`]).
            #[cfg(feature = "quiescence-search")]
            let declined = {
                let mut d = [0u8; MAX_PLAYERS_IN_GAME];
                if !matches!(proposed.action, GameAction::Play { .. })
                    && Self::player_has_uncashed_signalled_play(
                        &root_state,
                        static_data,
                        &truth_pov,
                        root_player,
                    )
                {
                    d[root_player] = 1;
                }
                d
            };
            #[cfg(not(feature = "quiescence-search"))]
            let declined = [0u8; MAX_PLAYERS_IN_GAME];
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
                declined,
                root_playable,
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
        };
        // Under the diagnostic feature, score candidates sequentially and print a banner per
        // root candidate. This keeps each candidate's inner subjective-sub-search dump
        // contiguous and attributable, instead of rayon-interleaving the 6k-line firehose
        // across candidates with no way to tell which root they belong to.
        #[cfg(feature = "search-diagnostic")]
        let mut nodes: Vec<ScoredNode> = candidates
            .into_vec()
            .into_iter()
            .map(|proposed| {
                println!(
                    "\n===================== ROOT CANDIDATE: {:?} (tech={}) =====================",
                    proposed.action, proposed.tech_name
                );
                eval_candidate(proposed)
            })
            .collect();
        #[cfg(not(feature = "search-diagnostic"))]
        let mut nodes: Vec<ScoredNode> = candidates
            .into_vec()
            .into_par_iter()
            .map(eval_candidate)
            .collect();
        nodes.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
        nodes
    }
}
