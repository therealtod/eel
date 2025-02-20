#[cfg(test)]
use mockall::automock;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, DeckCardsBitField, VariantCardId};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// Read-only view of the knowledge that a specific player has on the state of the game.
///
/// Covers public information (board state) as well as private information (teammate hands,
/// inferred identities, convention signals, etc.).
///
/// This trait is intentionally **read-only**: mutations go through
/// [`PlayerKnowledgeState`](super::player_knowledge_state::PlayerKnowledgeState) directly.
#[cfg_attr(test, automock)]
pub trait PlayerPOV {
    /// Get the "away value" of a card (how many plays away from being playable).
    fn away_value(&self, card_id: VariantCardId) -> u8;

    /// Get the known identity of a card from this POV, if fully determined.
    fn card_identity(&self, card_deck_index: CardDeckIndex) -> Option<VariantCardId>;

    fn valid_actions(&self) -> Vec<GameAction>;

    fn own_playable_cards(&self) -> DeckCardsBitField;

    fn is_playable(&self, card_deck_index: CardDeckIndex) -> bool;
    
    fn is_touched(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Returns true if the holder of this card knows its exact identity
    /// (i.e. it is in their `visible_cards`, meaning it was revealed to them via a clue or inference).
    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Returns true if the card is the last remaining copy (all other copies have been discarded).
    fn is_critical(&self, card_deck_index: CardDeckIndex) -> bool;

    fn player_on_turn_index(&self) -> PlayerIndex;

    fn table_state(&self) -> &TableState;

    fn static_data(&self) -> &StaticGameData;

    fn team_knowledge(&self) -> &TeamKnowledge;
}
