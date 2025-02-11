use std::fmt::{Display};
use crate::game::card::Empathy;

pub mod card;
pub mod core_game_state;
pub mod suit;
pub mod rank;

pub const MAX_UNIQUE_CARDS_IN_DECK: usize = 35;
pub const MAX_CARDS_IN_DECK: usize = 60;
pub const MAX_STACKS_IN_GAME: usize = 6;
pub const MAX_CLUE_TOKEN_COUNT: u8 = 8;
pub const MAX_CLUE_TYPES: usize = 2;
pub const MAX_CLUE_VALUES_PER_TYPE: usize = 6;
pub const ALL_CARDS_MASK:Empathy = Empathy::MAX;
mod game_state;
mod static_game_data;
mod variant;
mod clue_value;
mod game_error;
mod hand;
mod playing_stacks;
pub mod deck;
mod clue_token_bank;
mod action;
mod clue;
