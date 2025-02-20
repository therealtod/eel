use crate::game::{INITIAL_CLUE_TOKENS_COUNT, MAX_CLUE_TOKEN_COUNT};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ClueTokenBank {
    half_tokens_count: u8,
}

impl ClueTokenBank {
    pub const fn new(half_clue_tokens: u8) -> Self {
        ClueTokenBank { half_tokens_count: half_clue_tokens }
    }
    pub fn whole_clue_tokens_count(&self) -> u8 {
        self.half_tokens_count / 2
    }

    pub fn add_tokens(&mut self, tokens: u8) {
        self.half_tokens_count += tokens;
        if self.half_tokens_count > MAX_CLUE_TOKEN_COUNT * 2 {
            self.half_tokens_count = MAX_CLUE_TOKEN_COUNT * 2;
        }
    }

    pub fn use_token(&mut self) {
        if self.whole_clue_tokens_count() < 1 {
            panic!("Cannot use a token when the token count is lower than 1")
        }
        self.half_tokens_count -= 2
    }
}

impl Default for ClueTokenBank {
    fn default() -> Self {
        ClueTokenBank { half_tokens_count: INITIAL_CLUE_TOKENS_COUNT * 2 }
    }
}
