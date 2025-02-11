use crate::game::MAX_CLUE_TOKEN_COUNT;

pub struct ClueTokenBank {
    half_tokens_count: u8,
}

impl ClueTokenBank {
    pub fn get_usable_clue_tokens_count(&self) -> u8 {
        self.half_tokens_count/2
    }

    pub fn add_tokens(&mut self, tokens: u8) {
        self.half_tokens_count += tokens;
        if self.half_tokens_count > MAX_CLUE_TOKEN_COUNT*2 {
            self.half_tokens_count = MAX_CLUE_TOKEN_COUNT*2;
        }
    }

    pub fn use_token(&mut self) {
        if self.get_usable_clue_tokens_count() < 2 {
            panic!("Cannot use a token when the token count is lower than 1")
        }
        self.half_tokens_count -= 2
    }
}
