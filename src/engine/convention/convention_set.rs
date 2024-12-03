use crate::engine::convention::convention_tech::ConventionTech;

/// A complete convention framework agreed on by all players before the game.
///
/// Examples: H-Group, Referential Sieve, Reactor 1.0.
/// Frameworks are mutually exclusive — a game uses exactly one.
///
/// Techniques are stored in a single ordered list (by priority).
/// Convenience filter methods are provided for querying by action type.
pub trait ConventionSet: Sync {
    /// All techniques in priority order (lowest priority number first).
    fn techs(&self) -> &[Box<dyn ConventionTech>];
}
