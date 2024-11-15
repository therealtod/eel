#[derive(Debug, PartialEq)]
pub enum GameError {
    UnrecognizedCard,
    PeekedEmptyStack,
    NoClueTokens,
}
