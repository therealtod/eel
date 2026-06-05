use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use smallvec::SmallVec;

/// Inline capacity for candidate action lists. 20 covers all realistic Hanabi scenarios
/// (5-card hands × play+discard + a handful of convention-proposed clues) without spilling
/// to the heap on the hot recursive path.
pub const CANDIDATE_INLINE_CAP: usize = 20;

/// A proposed candidate action together with the tech that proposed it.
#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub action: GameAction,
    pub tech_name: &'static str,
    pub priority: u8,
}

/// The candidate action list produced and filtered during action selection. Backed by
/// inline storage so the typical case never touches the heap on the recursive hot path.
pub type CandidateActions = SmallVec<[ProposedAction; CANDIDATE_INLINE_CAP]>;

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

    /// Last-resort techniques consulted only when no primary tech has a candidate action.
    ///
    /// Returned in the order they should be tried (first match wins, mirroring the
    /// action-generation fallback chain). NOT sorted by `interpretation_priority`.
    fn fallback_techs(&self) -> &[Box<dyn ConventionTech>] {
        &[]
    }

    /// Per-clue baseline narrowing applied to the receiver's own empathy on touched cards.
    ///
    /// This is convention-wide knowledge that does not depend on which specific tech
    /// "explains" the clue. For H-Group this encodes the good-touch principle: every
    /// touched card is assumed to be eventually useful, so its identity narrows to the
    /// clue's empathy intersected with still-needed cards minus identities already
    /// clued in other hands. `focus` (the clue's focus card, if any) is used to resolve
    /// the no-duplication principle: when fewer good identities remain than cards touched,
    /// the focus has first claim and the remaining cards collapse to known trash.
    ///
    /// Returns `(card_deck_index, mask)` pairs, applied as baseline `narrow_inferred` on
    /// the receiver's `PlayerKnowledge`. Default: no narrowing.
    fn clue_receiver_baseline(
        &self,
        _clue: &Clue,
        _touched: &[CardDeckIndex],
        _focus: Option<CardDeckIndex>,
        _receiver: PlayerIndex,
        _table_state: &TableState,
        _static_data: &StaticGameData,
    ) -> Vec<(CardDeckIndex, VariantCardsBitField)> {
        Vec::new()
    }

    /// Convention-specific pruning of the deduplicated candidate action list, applied by the
    /// search layer after tech proposals are gathered and before move ordering.
    ///
    /// This is where a framework expresses priority and legality rules that decide *which of its
    /// own proposals survive* — e.g. dropping a chop discard when a known-trash discard exists, or
    /// forbidding a discard in a situation the convention considers a play-mandatory. Keeping it
    /// behind this hook is what stops framework-specific priority logic from leaking into the
    /// generic search layer. Default: no pruning.
    fn filter_candidate_actions(
        &self,
        _proposed: &mut CandidateActions,
        _pov: &dyn PlayerPOV,
        _is_early_game: bool,
    ) {
    }
}
