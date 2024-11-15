use crate::game::game_error::GameError;
use crate::game::{INITIAL_CLUE_TOKENS_COUNT, MAX_CLUE_TOKEN_COUNT};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ClueTokenBank {
    half_tokens_count: u8,
}

impl ClueTokenBank {
    pub const fn new(half_clue_tokens: u8) -> Self {
        ClueTokenBank {
            half_tokens_count: half_clue_tokens,
        }
    }

    pub const fn new_from_whole_tokens(clue_tokens: u8) -> Self {
        ClueTokenBank {
            half_tokens_count: clue_tokens * 2,
        }
    }
    pub fn whole_clue_tokens_count(&self) -> u8 {
        self.half_tokens_count / 2
    }

    pub fn half_tokens_count(&self) -> u8 {
        self.half_tokens_count
    }

    pub fn set_half_tokens(&mut self, value: u8) {
        self.half_tokens_count = value;
    }

    pub fn add_tokens(&mut self, tokens: u8) {
        self.half_tokens_count += tokens;
        if self.half_tokens_count > MAX_CLUE_TOKEN_COUNT * 2 {
            self.half_tokens_count = MAX_CLUE_TOKEN_COUNT * 2;
        }
    }

    pub fn use_token(&mut self) -> Result<(), GameError> {
        if self.whole_clue_tokens_count() < 1 {
            return Err(GameError::NoClueTokens);
        }
        self.half_tokens_count -= 2;
        Ok(())
    }
}

impl Default for ClueTokenBank {
    fn default() -> Self {
        ClueTokenBank {
            half_tokens_count: INITIAL_CLUE_TOKENS_COUNT * 2,
        }
    }
}
