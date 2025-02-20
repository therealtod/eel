/// Represents the action of giving a clue in a game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Clue {
    pub clue_type: u8,
    pub clue_value: u8,
}
