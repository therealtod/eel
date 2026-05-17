use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, CardIdentityMask, DeckCardsBitField, VariantCardId};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
#[cfg(test)]
use mockall::automock;

/// Read-only view of the knowledge that a specific player has on the state of the game.
///
/// Covers public information (board state) as well as private information (teammate hands,
/// inferred identities, convention signals, etc.).
///
/// This trait is intentionally **read-only**: mutations go through
/// [`PlayerKnowledgeState`](super::player_knowledge::PlayerKnowledgeState) directly.
#[cfg_attr(test, automock)]
pub trait PlayerPOV {
    /// Get the player's own player index
    fn player_index(&self) -> PlayerIndex;

    /// Get the "away value" of a card (how many plays away from being playable).
    fn away_value(&self, card_id: VariantCardId) -> Option<u8>;

    /// Get the known identity of a card from this POV, if fully determined.
    fn card_identity(&self, card_deck_index: CardDeckIndex) -> Option<VariantCardId>;

    fn own_playable_cards(&self) -> DeckCardsBitField;

    fn is_playable(&self, card_deck_index: CardDeckIndex) -> bool;

    fn is_touched(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Returns true if the holder of this card knows its exact identity
    /// (i.e. it is in their `visible_cards`, meaning it was revealed to them via a clue or inference).
    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Returns true if the card is the last remaining copy (all other copies have been discarded).
    fn is_critical(&self, card_deck_index: CardDeckIndex) -> bool;

    fn is_critical_card_id(&self, variant_card_id: VariantCardId) -> bool;

    fn active_player_index(&self) -> PlayerIndex;

    fn table_state(&self) -> &TableState;

    fn static_data(&self) -> &StaticGameData;

    fn team_knowledge(&self) -> &TeamKnowledge;

    /// Reconstruct the POV of `player_index` using the knowledge this player has about them.
    ///
    /// The returned view uses `team_knowledge().player(player_index)` as the knowledge source,
    /// so all empathy, signals, and visible-card information reflect what `player_index` knows
    /// (as tracked by the current observer), not what the current observer knows.
    fn as_player_pov(&self, player_index: PlayerIndex) -> LightweightPlayerPOV<'_> {
        LightweightPlayerPOV::new(
            player_index,
            self.team_knowledge().player(player_index),
            self.team_knowledge(),
            self.table_state(),
            self.static_data(),
        )
    }

    /// Returns true if the card is known to be trash from this POV
    fn is_known_trash(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Get the empathy that this player has for the given card.
    fn empathy(&self, card_deck_index: CardDeckIndex) -> CardIdentityMask;

    #[must_use]
    fn valid_actions(&self) -> Vec<GameAction>;

    #[must_use]
    fn gotten_cards(&self) -> CardIdentityMask;

    #[must_use]
    fn is_gotten(&self, variant_card_id: VariantCardId) -> bool;
}
