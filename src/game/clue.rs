use crate::game::clue_type::ClueType;

/// Represents the action of giving a clue in a game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Clue {
    pub clue_type: ClueType,
    pub clue_value: u8,
}
