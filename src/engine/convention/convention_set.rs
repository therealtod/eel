use crate::engine::convention::convention_tech::ConventionTech;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

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

    /// Per-clue baseline narrowing applied to the receiver's own empathy on touched cards.
    ///
    /// This is convention-wide knowledge that does not depend on which specific tech
    /// "explains" the clue. For H-Group this encodes the good-touch principle: every
    /// touched card is assumed to be eventually useful, so its identity narrows to the
    /// clue's empathy intersected with still-needed cards minus identities already
    /// clued in other hands.
    ///
    /// Returns `(card_deck_index, mask)` pairs, applied as baseline `narrow_inferred` on
    /// the receiver's `PlayerKnowledge`. Default: no narrowing.
    fn clue_receiver_baseline(
        &self,
        _clue: &Clue,
        _touched: &[CardDeckIndex],
        _receiver: PlayerIndex,
        _table_state: &TableState,
        _static_data: &StaticGameData,
    ) -> Vec<(CardDeckIndex, VariantCardsBitField)> {
        Vec::new()
    }
}
