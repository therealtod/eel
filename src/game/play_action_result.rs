/// The result of the action of a player trying to play a card on the stacks
pub struct PlayActionResult {
    /// Whether the card has been successfully played on the stacks
    pub success: bool,
    /// The amount of half clue tokens that get awarded to the team after the card is played 
    /// (successfully)
    pub bonus_half_clue_tokens: u8,
}
