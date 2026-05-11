use crate::game::action::game_action::GameAction;

/// Score representing how good a game state is after performing an action.
/// Higher is better.
pub type Score = f64;

/// A node in the decision tree: an action and its score (propagated up from the leaf).
#[derive(Debug, Clone)]
pub struct ScoredNode {
    /// The game action this node represents.
    pub action: GameAction,
    /// Best leaf score reachable from this node's subtree.
    pub total_score: Score,
    /// Name of the tech that proposed this action (e.g. `"SimpleFinesse"`).
    pub tech_name: &'static str,
    /// Principal variation: the full sequence of actions from this node's action through to
    /// the leaf that produced `total_score` (root action first, then subsequent best actions).
    pub line: Vec<GameAction>,
}

impl ScoredNode {
    /// Construct a leaf node with the given action, score, proposing tech, and principal variation.
    pub fn leaf(action: GameAction, score: Score, tech_name: &'static str, line: Vec<GameAction>) -> Self {
        ScoredNode {
            action,
            total_score: score,
            tech_name,
            line,
        }
    }
}

