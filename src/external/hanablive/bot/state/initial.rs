use crate::external::hanablive::bot::state::common::CommonState;

pub struct InitialState {
    pub common_state: CommonState,
    pub username: String,
    pub password: String,
}

impl InitialState {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            common_state: CommonState::default(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}
