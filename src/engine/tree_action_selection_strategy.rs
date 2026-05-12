use rayon::prelude::*;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::decision_tree::{LineStep, Score, ScoredNode};
use crate::engine::evaluator::{DefaultEvaluator, Evaluator, ScoreBreakdown};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::action::game_action::GameAction;
use crate::game::static_game_data::StaticGameData;

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
    pub fn candidate_actions_with_provenance(
        pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> Vec<ProposedAction> {
        let has_clue_tokens = pov.table_state().clue_token_bank.whole_clue_tokens_count() > 0;
        let mut proposed: Vec<ProposedAction> = convention_set
            .techs()
            .iter()
            .flat_map(|tech| {
                let tech_name = tech.name();
                let priority = tech.interpretation_priority();
                tech.game_actions(pov)
                    .into_iter()
                    .filter(move |a| has_clue_tokens || !matches!(a, GameAction::Clue { .. }))
                    .map(move |action| ProposedAction { action, tech_name, priority })
            })
            .collect();

        if proposed.is_empty() {
            proposed = pov
                .valid_actions()
                .into_iter()
                .map(|action| ProposedAction { action, tech_name: "fallback", priority: u8::MAX })
                .collect();
        }

        // Dedup by action, keeping the first (highest-priority) tech that proposed it.
        // Vec scan is faster than HashSet for the small candidate counts seen in practice.
        let mut seen: Vec<GameAction> = Vec::with_capacity(proposed.len());
        proposed.retain(|p| {
            if seen.contains(&p.action) {
                false
            } else {
                seen.push(p.action.clone());
                true
            }
        });

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
    ) -> ScoreBreakdown {
        evaluator.score_breakdown(
            &state.table_state,
            &state.static_data(),
            &state.team_knowledge,
        )
    }

    /// Per-action bonus applied along the search path. Currently only clue actions
    /// produce a non-zero value (good-touch penalty and clue precision reward).
    fn immediate_action_bonus(
        action: &GameAction,
        evaluator: &dyn Evaluator,
        state_after: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
    ) -> Score {
        if let GameAction::Clue {
            touched_card_deck_indexes,
            player_index,
            ..
        } = action
        {
            evaluator.clue_precision_bonus(
                touched_card_deck_indexes,
                *player_index,
                static_data,
                &state_after.team_knowledge,
                state_after.table_state(),
            )
        } else {
            0.0
        }
    }

    /// Recursively compute the best leaf score reachable from `state` within `depth` more turns.
    ///
    /// Returns `(score, pv)` where `pv` is the **principal variation** — the sequence of actions
    /// taken at each ply from the current node down to the leaf that produced `score`. Callers
    /// prepend their own action to build the full line stored in [`ScoredNode::line`].
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
    /// No alpha-beta pruning is performed: every ply maximises, so there is no min-parent that
    /// would reject a value ≥ alpha — truncating a subtree's value would silently change which
    /// root candidate wins. A real prune would require a state-derived upper bound on the
    /// subtree, which we don't currently compute.
    fn best_score_at_depth(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
    ) -> (Score, Vec<LineStep>, ScoreBreakdown) {
        if depth == 0 || state.table_state.is_terminal(static_data) {
            let breakdown = Self::leaf_breakdown(evaluator, state);
            tracing::trace!(
                target: "eel::search",
                depth = 0,
                terminal = state.table_state.is_terminal(static_data),
                leaf = %breakdown,
                "leaf_reached",
            );
            return (breakdown.total, vec![], breakdown);
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
        let mut best_pv: Vec<LineStep> = vec![];
        let mut best_breakdown: Option<ScoreBreakdown> = None;
        for proposed in candidates {
            let mut next = state.clone();
            next.apply(&proposed.action, convention_set);
            next.advance_turn();
            let immediate =
                Self::immediate_action_bonus(&proposed.action, evaluator, &next, static_data);
            let (subtree_score, rest, leaf_bd) = Self::best_score_at_depth(
                &next,
                static_data,
                convention_set,
                evaluator,
                depth - 1,
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
                best_pv = Vec::with_capacity(rest.len() + 1);
                best_pv.push(LineStep {
                    action: proposed.action,
                    tech_name: proposed.tech_name,
                    immediate_bonus: immediate,
                });
                best_pv.extend(rest);
                best_breakdown = Some(leaf_bd);
            }
        }
        // `best_breakdown` is set whenever the candidate loop ran. If no candidates were
        // produced (extremely rare — the fallback in `candidate_actions_with_provenance`
        // makes this near-impossible), score the current state as the leaf.
        let breakdown = best_breakdown.unwrap_or_else(|| Self::leaf_breakdown(evaluator, state));
        (best, best_pv, breakdown)
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
        let depth = static_data.number_of_players as usize;
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
        let span = tracing::debug_span!(
            target: "eel::search",
            "scored_actions",
            player = player_pov.table_state().active_player_index,
            candidates = candidates.len(),
        );
        let mut nodes: Vec<ScoredNode> = candidates
            .into_par_iter()
            .map(|proposed| {
                let _guard = span.clone().entered();
                let mut next = root_state.clone();
                next.apply(&proposed.action, convention_set);
                next.advance_turn();
                let immediate_bonus =
                    Self::immediate_action_bonus(&proposed.action, evaluator, &next, static_data);
                let (leaf_score, pv, leaf_breakdown) = Self::best_score_at_depth(
                    &next,
                    static_data,
                    convention_set,
                    evaluator,
                    depth - 1,
                );
                let total = leaf_score + immediate_bonus;
                let mut line = Vec::with_capacity(pv.len() + 1);
                line.push(LineStep {
                    action: proposed.action.clone(),
                    tech_name: proposed.tech_name,
                    immediate_bonus,
                });
                line.extend(pv);
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
                ScoredNode::leaf(proposed.action, total, proposed.tech_name, line, leaf_breakdown)
            })
            .collect();
        nodes.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
        nodes
    }
}
