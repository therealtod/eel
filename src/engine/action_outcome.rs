/// Outcome of applying an action, carrying search-relevant metadata.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActionOutcome {
    /// True when the play succeeded, but its exact stack assignment was deferred
    /// (known-playable but multiple candidate identities).
    pub is_phantom_play: bool,
}
