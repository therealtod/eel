use crate::external::hanablive::bot::state::common::CommonState;

pub struct InitialState {
    pub common_state: CommonState,
    pub username: String,
    pub password: String,
}

impl InitialState {
    pub fn new(username: String, password: String) -> Self {
        Self {
            common_state: CommonState::default(),
            username,
            password,
        }
    }
}
