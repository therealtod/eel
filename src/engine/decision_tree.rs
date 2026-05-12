use crate::engine::evaluator::ScoreBreakdown;
use crate::game::action::game_action::GameAction;
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;

/// Score representing how good a game state is after performing an action.
/// Higher is better.
pub type Score = f64;

/// One step along a principal variation.
///
/// `immediate_bonus` is the per-action contribution applied along the search line
/// (currently non-zero only for clues: `clue_precision_bonus − good_touch_penalty * bad_touches`).
/// Summing `immediate_bonus` across the line and adding it to `leaf_breakdown.total`
/// reproduces [`ScoredNode::total_score`].
#[derive(Debug, Clone)]
pub struct LineStep {
    pub action: GameAction,
    pub tech_name: &'static str,
    pub immediate_bonus: Score,
}

/// A node in the decision tree: an action and its score (propagated up from the leaf).
#[derive(Debug, Clone)]
pub struct ScoredNode {
    /// The game action this node represents.
    pub action: GameAction,
    /// Best leaf score reachable from this node's subtree, plus the sum of immediate
    /// bonuses along the principal variation.
    pub total_score: Score,
    /// Name of the tech that proposed this action (e.g. `"SimpleFinesse"`).
    pub tech_name: &'static str,
    /// Principal variation: the full sequence of (action, proposing tech, immediate bonus)
    /// triples from this node's action through to the leaf that produced `total_score`.
    pub line: Vec<LineStep>,
    /// Per-term breakdown of the leaf state reached at the end of `line`.
    pub leaf_breakdown: ScoreBreakdown,
}

impl ScoredNode {
    /// Construct a leaf node with the given action, score, proposing tech, principal variation,
    /// and leaf-state score breakdown.
    pub fn leaf(
        action: GameAction,
        score: Score,
        tech_name: &'static str,
        line: Vec<LineStep>,
        leaf_breakdown: ScoreBreakdown,
    ) -> Self {
        ScoredNode {
            action,
            total_score: score,
            tech_name,
            line,
            leaf_breakdown,
        }
    }
}

fn fmt_action(action: &GameAction, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match action {
        GameAction::Play {
            player_index,
            card_deck_index,
            ..
        } => write!(f, "P{} Play deck#{}", player_index, card_deck_index),
        GameAction::Discard {
            player_index,
            card_deck_index,
            ..
        } => write!(f, "P{} Discard deck#{}", player_index, card_deck_index),
        GameAction::Clue {
            player_index,
            clue: Clue { clue_type, clue_value },
            touched_card_deck_indexes,
            ..
        } => {
            let kind = match clue_type {
                ClueType::Color => "color",
                ClueType::Rank => "rank",
            };
            write!(
                f,
                "P{} Clue {}={} touches {:?}",
                player_index,
                kind,
                clue_value,
                touched_card_deck_indexes.as_slice()
            )
        }
        GameAction::Draw {
            player_index,
            card_deck_index,
        } => write!(f, "P{} Draw deck#{}", player_index, card_deck_index),
    }
}

impl std::fmt::Display for LineStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_action(&self.action, f)?;
        write!(f, " [{}] imm={:+.2}", self.tech_name, self.immediate_bonus)
    }
}

impl std::fmt::Display for ScoredNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let imm_sum: f64 = self.line.iter().map(|s| s.immediate_bonus).sum();
        writeln!(
            f,
            "total={:.2}  (leaf={:.2}  +Σimm={:+.2})",
            self.total_score, self.leaf_breakdown.total, imm_sum
        )?;
        for (i, step) in self.line.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, step)?;
        }
        write!(f, "  leaf: {}", self.leaf_breakdown)
    }
}
