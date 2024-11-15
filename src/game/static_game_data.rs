use crate::game::variant::Variant;

/// Data that stays invariant through the whole game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticGameData {
    /// Number of players participating in the game
    pub number_of_players: u8,
    /// The [Variant] selected for the game
    pub variant: Variant,
}
