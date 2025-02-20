use rayon::prelude::*;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::decision_tree::{DecisionTree, Score, ScoredNode};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::player_pov_view::PlayerPOVView;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;use crate::game::action::game_action::GameAction;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

pub struct TreeActionSelectionStrategy;

/// Cloneable snapshot of the game state used during tree search.
#[derive(Clone)]
struct SearchState {
    table_state: TableState,
    team_knowledge: TeamKnowledge,
}

impl SearchState {
    fn pov<'a>(&'a self, static_data: &'a StaticGameData) -> PlayerPOVView<'a> {
        let player_index = self.table_state.player_on_turn_index;
        PlayerPOVView::new(
            player_index,
            self.team_knowledge.player(player_index),
            &self.team_knowledge,
            &self.table_state,
            static_data,
        )
    }

    /// Apply an action and advance the turn to the next player.
    fn apply(&mut self, action: &GameAction, static_data: &StaticGameData) {
        let num_players = static_data.number_of_players as usize;
        match action {
            GameAction::Play { card_deck_index, .. } => {
                self.table_state.update_with_play_action(*card_deck_index);
                let p = self.table_state.player_on_turn_index;
                self.team_knowledge.player_mut(p).own_hand &= !(1u64 << card_deck_index);
            }
            GameAction::Discard { card_deck_index, .. } => {
                self.table_state.update_with_discard_action(*card_deck_index, static_data);
                let p = self.table_state.player_on_turn_index;
                self.team_knowledge.player_mut(p).own_hand &= !(1u64 << card_deck_index);
            }
            GameAction::Clue { touched_card_deck_indexes, clue, player_index, .. } => {
                self.table_state.update_with_clue_action(
                    touched_card_deck_indexes.clone(),
                    clue.clone(),
                    *player_index,
                    static_data,
                );
            }
            GameAction::Draw { card_deck_index, player_index } => {
                self.table_state.update_with_draw_action(*card_deck_index);
                self.team_knowledge.player_mut(*player_index).own_hand |= 1 << card_deck_index;
            }
        }
        self.table_state.player_on_turn_index =
            (self.table_state.player_on_turn_index + 1) % num_players;
    }
}

impl TreeActionSelectionStrategy {
    fn candidate_actions(pov: &dyn PlayerPOV, convention_set: &dyn ConventionSet) -> Vec<GameAction> {
        let mut actions: Vec<GameAction> = convention_set
            .techs()
            .iter()
            .flat_map(|tech| tech.game_actions(pov))
            .collect();
        if actions.is_empty() {
            actions = pov.valid_actions();
        }
        actions.dedup();
        actions
    }

    /// Dummy leaf heuristic — replace with a real evaluation later.
    fn score(action: &GameAction) -> Score {
        match action {
            GameAction::Play { .. } => 10,
            GameAction::Clue { .. } => 5,
            GameAction::Discard { .. } => 1,
            GameAction::Draw { .. } => 0,
        }
    }

    /// Recursively compute the best score reachable from `state` within `depth` more turns.
    /// Returns 0 when depth is exhausted (no further look-ahead).
    fn best_score_at_depth(
        state: &SearchState,
        static_data: &StaticGameData,
        convention_set: &dyn ConventionSet,
        depth: usize,
    ) -> Score {
        if depth == 0 {
            return 0;
        }
        let pov = state.pov(static_data);
        let candidates = Self::candidate_actions(&pov, convention_set);
        candidates
            .into_iter()
            .map(|action| {
                let immediate = Self::score(&action);
                let mut next = state.clone();
                next.apply(&action, static_data);
                immediate + Self::best_score_at_depth(&next, static_data, convention_set, depth - 1)
            })
            .max()
            .unwrap_or(0)
    }
}

impl ActionSelectionStrategy for TreeActionSelectionStrategy {
    fn select_active_player_action(
        &self,
        player_pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> GameAction {
        let static_data = player_pov.static_data();
        let depth = static_data.number_of_players as usize;

        let root_state = SearchState {
            table_state: player_pov.table_state().clone(),
            team_knowledge: player_pov.team_knowledge().clone(),
        };

        let candidates = Self::candidate_actions(player_pov, convention_set);

        // Evaluate each root candidate in parallel.
        let scored_nodes: Vec<ScoredNode> = candidates
            .into_par_iter()
            .map(|action| {
                let immediate = Self::score(&action);
                let mut next = root_state.clone();
                next.apply(&action, static_data);
                // Search the remaining depth - 1 turns (we already consumed one turn above).
                let best_child = Self::best_score_at_depth(
                    &next,
                    static_data,
                    convention_set,
                    depth - 1,
                );
                ScoredNode::with_best_child(action, immediate, best_child)
            })
            .collect();

        let tree = DecisionTree::new(scored_nodes);
        tree.best_action()
            .cloned()
            .unwrap_or_else(|| {
                player_pov
                    .valid_actions()
                    .into_iter()
                    .next()
                    .expect("no valid actions available for the active player")
            })
    }
}
