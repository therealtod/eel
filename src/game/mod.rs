pub type SlotIndex = usize;

// Hanabi game constants
pub const MAX_PLAYERS_IN_GAME: usize = 6;
pub const MAX_HAND_SIZE: usize = 5;
pub const MAX_UNIQUE_CARDS_IN_DECK: usize = 35;
pub const MAX_CARDS_IN_DECK: usize = 60;
pub const MAX_SUITS_IN_GAME: usize = 6;
pub const MAX_CARDS_PER_STACK: usize = 5;
pub const MAX_CLUE_TOKEN_COUNT: u8 = 8;
pub const MAX_CLUE_TYPES_IN_VARIANT: usize = 2;
pub const MAX_CLUE_VALUES_PER_TYPE: usize = 6;
pub const INITIAL_CLUE_TOKENS_COUNT: u8 = 8;

pub mod action;
pub mod card;
pub mod clue;
pub mod clue_token_bank;
pub mod clue_type;
pub mod deck;
pub mod game_error;
pub(crate) mod hand;
pub mod playing_stacks;
pub mod state;
pub mod static_game_data;
pub mod variant;
