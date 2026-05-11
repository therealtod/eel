use rayon::prelude::*;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::decision_tree::{Score, ScoredNode};
use crate::engine::evaluator::{DefaultEvaluator, Evaluator};
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

    fn node_score(evaluator: &dyn Evaluator, state: &KnowledgeAwareGameState) -> Score {
        evaluator.score(
            &state.table_state,
            &state.static_data(),
            &state.team_knowledge,
        )
    }

    /// Recursively compute the best leaf score reachable from `state` within `depth` more turns.
    ///
    /// Returns `(score, pv)` where `pv` is the **principal variation** — the sequence of actions
    /// taken at each ply from the current node down to the leaf that produced `score`. Callers
    /// prepend their own action to build the full line stored in [`ScoredNode::line`].
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
    /// `alpha` is the best score the parent has already found; subtrees that cannot beat it are pruned.
    fn best_score_at_depth(
        state: &KnowledgeAwareGameState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        evaluator: &dyn Evaluator,
        depth: usize,
        alpha: Score,
    ) -> (Score, Vec<GameAction>) {
        if depth == 0 || state.table_state.is_terminal(static_data) {
            return (Self::node_score(evaluator, state), vec![]);
        }

        let pov = state.player_pov(state.table_state.active_player_index);
        let candidates = Self::candidate_actions_with_provenance(&pov, convention_set)
            .into_iter()
            .map(|p| p.action);
        let mut best = f64::NEG_INFINITY;
        let mut best_pv: Vec<GameAction> = vec![];
        for action in candidates {
            let mut next = state.clone();
            next.apply(&action, convention_set);
            next.advance_turn();
            let (score, rest) = Self::best_score_at_depth(
                &next,
                static_data,
                convention_set,
                evaluator,
                depth - 1,
                best,
            );
            if score > best {
                best = score;
                best_pv = Vec::with_capacity(rest.len() + 1);
                best_pv.push(action);
                best_pv.extend(rest);
            }
            // Cooperative Hanabi has no adversary, so a single alpha cutoff suffices:
            // if we've found something at least as good as what the parent already has,
            // the parent won't choose this subtree regardless of remaining siblings.
            if best >= alpha {
                return (best, best_pv);
            }
        }
        (best, best_pv)
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
                let (leaf_score, pv) = Self::best_score_at_depth(
                    &next,
                    static_data,
                    convention_set,
                    evaluator,
                    depth - 1,
                    f64::NEG_INFINITY,
                );
                let empathy_bonus: Score = if let GameAction::Clue {
                    touched_card_deck_indexes,
                    player_index,
                    ..
                } = &proposed.action
                {
                    evaluator.clue_precision_bonus(
                        touched_card_deck_indexes,
                        *player_index,
                        static_data,
                        &next.team_knowledge,
                        next.table_state(),
                    )
                } else {
                    0.0
                };
                let total = leaf_score + empathy_bonus;
                let mut line = Vec::with_capacity(pv.len() + 1);
                line.push(proposed.action.clone());
                line.extend(pv);
                tracing::debug!(
                    target: "eel::search",
                    action = ?proposed.action,
                    tech = proposed.tech_name,
                    priority = proposed.priority,
                    leaf_score,
                    empathy_bonus,
                    total,
                    line = ?line,
                    "candidate_scored",
                );
                ScoredNode::leaf(proposed.action, total, proposed.tech_name, line)
            })
            .collect();
        nodes.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
        nodes
    }
}
