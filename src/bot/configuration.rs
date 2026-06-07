use std::time::Duration;

use crate::engine::convention::hgroup::h_group_convention_set::HGroupLevel;

#[derive(Debug, Clone)]
pub struct Configuration {
    /// Maximum number of consecutive reconnect attempts after a disconnect before giving up.
    pub max_reconnect_attempts: u32,
    /// Backoff before the first reconnect attempt. Doubles after each failed attempt, capped at
    /// [`Configuration::reconnect_max_backoff`].
    pub reconnect_initial_backoff: Duration,
    /// Upper bound on the (exponentially growing) reconnect backoff.
    pub reconnect_max_backoff: Duration,
    /// H-Group convention level the bot plays at.
    pub hgroup_level: HGroupLevel,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            max_reconnect_attempts: 10,
            reconnect_initial_backoff: Duration::from_secs(1),
            reconnect_max_backoff: Duration::from_secs(30),
            hgroup_level: HGroupLevel::One,
        }
    }
}

impl Configuration {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
