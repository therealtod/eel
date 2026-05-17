use rayon::prelude::*;
use smallvec::SmallVec;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::decision_tree::{LineStep, Score, ScoredNode};
use crate::engine::evaluator::{DefaultEvaluator, Evaluator, ScoreBreakdown};
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::action::game_action::GameAction;
use crate::game::static_game_data::StaticGameData;

/// Inline capacity for candidate action lists. 20 covers all realistic Hanabi scenarios
/// (5-card hands × play+discard + a handful of convention-proposed clues) without spilling
/// to the heap on the hot recursive path.
const CANDIDATE_INLINE_CAP: usize = 20;

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
            proposed = pov
                .valid_actions()
                .into_iter()
                .map(|action| ProposedAction {
                    action,
                    tech_name: "fallback",
                    priority: u8::MAX,
                })
                .collect();
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
    ) -> ScoreBreakdown {
        evaluator.score_breakdown(state, truth)
    }

    /// Per-action bonus applied along the search path.
    ///
    /// Sums two terms:
    /// - `clue_precision_bonus` for clue actions (good-touch penalty + precision reward),
    ///   evaluated from the post-action state.
    /// - `signal_ignored_penalty` charged when the actor took a non-Play (or wrong-Play)
    ///   action while holding an active `Signal::Play` on an untouched own-hand card.
    ///   Evaluated from the pre-action state, where the actor is the player who chose
    ///   the action (i.e. `state_before.active_player_index`).
    fn immediate_action_bonus(
        action: &GameAction,
        evaluator: &dyn Evaluator,
        state_before: &KnowledgeAwareGameState,
        state_after: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        truth: &dyn PlayerPOV,
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
                truth,
                static_data,
                &state_after.team_knowledge,
                state_after.table_state(),
            )
        } else {
            0.0
        };
        let play_bonus = evaluator.play_progress_bonus(action, state_before, state_after);
        clue_bonus + signal_penalty + play_bonus
    }

    /// Recursively compute the best leaf score reachable from `state` within `depth` more turns.
    ///
    /// The principal variation is written into `pv_table` at row `depth`; callers read it via
    /// [`PvTable::pv_at`] after this function returns and prepend their own action.
    ///
    /// The returned score is the leaf evaluation **plus** the sum of per-action immediate bonuses
    /// (currently `clue_precision_bonus`) accumulated along the chosen line, so that good-touch
    /// violations and clue precision are visible at every ply, not just at the root.
    ///
    /// # Search model
    ///
    /// This is a **cooperative maximising search** — there is no adversary. At every ply the
    /// current player generates candidate actions from *their own* POV (i.e. using only the
    /// information available to them), applies the best-looking one, and the resulting game
    /// state is scored objectively. The best leaf score then bubbles back up to rank the root
    /// candidates chosen by the actor.
    ///
    /// Concretely, for a root action `a` chosen by the actor:
    /// 1. `a` is applied and the turn advances to the next player.
    /// 2. Each subsequent player also picks their best action (from their POV).
    /// 3. The leaf state is evaluated with a shared objective score (stacks, clue tokens, …).
    /// 4. That score propagates back to rank `a` against the other root candidates.
    ///
    /// The question being answered is therefore: *"how good does the game get if everyone plays
    /// optimally after I do this?"* — not a subjective per-player utility.
    ///
    /// Branch-and-bound pruning is applied: before recursing into each candidate, the evaluator's
    /// `upper_bound` is queried for the post-action state at the remaining depth. If that ceiling
    /// cannot beat the best score already secured at this node, the candidate is skipped. Once
    /// `best` reaches the node's own ceiling, the loop exits early.
    fn best_score_at_depth(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
        pv_table: &mut PvTable,
        truth: &dyn PlayerPOV,
    ) -> (Score, ScoreBreakdown) {
        if depth == 0 || state.table_state.is_terminal(static_data) {
            let breakdown = Self::leaf_breakdown(evaluator, state, truth);
            tracing::trace!(
                target: "eel::search",
                depth = 0,
                terminal = state.table_state.is_terminal(static_data),
                leaf = %breakdown,
                "leaf_reached",
            );
            // Clear so the parent sees an empty child PV when calling set_pv.
            pv_table.rows[depth].clear();
            return (breakdown.total, breakdown);
        }

        let active = state.table_state.active_player_index;
        let pov = state.player_pov(active);
        let candidates = Self::candidate_actions_with_provenance(&pov, convention_set);
        let span = tracing::trace_span!(
            target: "eel::search",
            "search_ply",
            depth,
            player = active,
            candidates = candidates.len(),
        );
        let _guard = span.enter();
        let mut best = f64::NEG_INFINITY;
        let mut best_breakdown: Option<ScoreBreakdown> = None;
        // Precompute the theoretical ceiling for this node so we can exit early once
        // we've already found a line that cannot be improved upon.
        let node_ceiling = evaluator.upper_bound(state.table_state(), static_data, depth);
        for proposed in candidates {
            let mut next = state.clone();
            next.apply(&proposed.action, convention_set, truth);
            next.advance_turn();
            let immediate = Self::immediate_action_bonus(
                &proposed.action,
                evaluator,
                state,
                &next,
                static_data,
                truth,
            );
            // Branch-and-bound: skip this candidate if its optimistic upper bound cannot
            // beat the best score we have already secured at this node.
            let candidate_ceiling =
                evaluator.upper_bound(next.table_state(), static_data, depth - 1) + immediate;
            if candidate_ceiling <= best {
                tracing::trace!(
                    target: "eel::search",
                    action = ?proposed.action,
                    tech = proposed.tech_name,
                    candidate_ceiling,
                    best,
                    "candidate_pruned",
                );
                continue;
            }
            let (subtree_score, leaf_bd) = Self::best_score_at_depth(
                &next,
                static_data,
                convention_set,
                evaluator,
                depth - 1,
                pv_table,
                truth,
            );
            let score = subtree_score + immediate;
            let improved = score > best;
            tracing::trace!(
                target: "eel::search",
                action = ?proposed.action,
                tech = proposed.tech_name,
                immediate,
                subtree_score,
                score,
                improved,
                "candidate_evaluated",
            );
            if improved {
                best = score;
                pv_table.set_pv(
                    depth,
                    LineStep {
                        action: proposed.action,
                        tech_name: proposed.tech_name,
                        immediate_bonus: immediate,
                    },
                );
                best_breakdown = Some(leaf_bd);
                // Early exit: already reached the theoretical ceiling for this node.
                if best >= node_ceiling {
                    break;
                }
            }
        }
        // `best_breakdown` is set whenever the candidate loop ran. If no candidates were
        // produced (extremely rare — the fallback in `candidate_actions_with_provenance`
        // makes this near-impossible), score the current state as the leaf.
        let breakdown =
            best_breakdown.unwrap_or_else(|| Self::leaf_breakdown(evaluator, state, truth));
        (best, breakdown)
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
                // Truth POV: the root searcher, reconstructed against root_state. Held fixed
                // across recursion so play resolution uses the searcher's visible-card truth.
                let root_knowledge = root_state.team_knowledge.player(root_player).clone();
                let truth_pov = LightweightPlayerPOV::new(
                    root_player,
                    &root_knowledge,
                    &root_state.team_knowledge,
                    root_state.table_state(),
                    static_data,
                );
                let mut next = root_state.clone();
                next.apply(&proposed.action, convention_set, &truth_pov);
                next.advance_turn();
                let immediate_bonus = Self::immediate_action_bonus(
                    &proposed.action,
                    evaluator,
                    &root_state,
                    &next,
                    static_data,
                    &truth_pov,
                );
                // Allocate once per root candidate (outside the recursive hot path).
                let mut pv_table = PvTable::new(subtree_depth);
                let (leaf_score, leaf_breakdown) = Self::best_score_at_depth(
                    &next,
                    static_data,
                    convention_set,
                    evaluator,
                    subtree_depth,
                    &mut pv_table,
                    &truth_pov,
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
