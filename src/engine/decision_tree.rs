use crate::game::action::game_action::GameAction;

/// Score representing how good a game state is after performing an action.
/// Higher is better.
pub type Score = i32;

/// A node in the decision tree: an action, its immediate score, and the best
/// score reachable from the resulting state (propagated up from children).
#[derive(Debug, Clone)]
pub struct ScoredNode {
    pub action: GameAction,
    /// Score of this action alone (leaf heuristic).
    pub immediate_score: Score,
    /// Best total score reachable from this node's subtree (immediate + best child).
    pub total_score: Score,
}

impl ScoredNode {
    pub fn leaf(action: GameAction, score: Score) -> Self {
        ScoredNode { action, immediate_score: score, total_score: score }
    }

    pub fn with_best_child(action: GameAction, immediate_score: Score, best_child_score: Score) -> Self {
        ScoredNode {
            action,
            immediate_score,
            total_score: immediate_score + best_child_score,
        }
    }
}

/// A decision tree whose root nodes are the candidate actions for the active player.
/// Each root node may have a `total_score` that incorporates look-ahead.
pub struct DecisionTree {
    pub nodes: Vec<ScoredNode>,
}

impl DecisionTree {
    pub fn new(nodes: Vec<ScoredNode>) -> Self {
        DecisionTree { nodes }
    }

    /// Return the action with the highest total score, or `None` if the tree is empty.
    pub fn best_action(&self) -> Option<&GameAction> {
        self.nodes
            .iter()
            .max_by_key(|n| n.total_score)
            .map(|n| &n.action)
    }
}
