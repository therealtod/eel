use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::MAX_PLAYERS_IN_GAME;

/// Collective representation of what all players on the team know.
///
/// Stores one [`PlayerKnowledgeState`] per player in a fixed-size array
/// (sized to `MAX_PLAYERS_IN_GAME`) for stack-friendly, clone-cheap search nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamKnowledge {
    player_knowledge: [PlayerKnowledgeState; MAX_PLAYERS_IN_GAME],
    num_players: usize,
}

impl TeamKnowledge {
    pub fn new(num_players: usize) -> Self {
        let mut player_knowledge = std::array::from_fn(|_| PlayerKnowledgeState::empty());
        for i in 0..num_players {
            player_knowledge[i] = PlayerKnowledgeState::new(i);
        }
        TeamKnowledge {
            player_knowledge,
            num_players,
        }
    }

    /// Get a reference to a specific player's knowledge.
    pub fn player(&self, player_index: usize) -> &PlayerKnowledgeState {
        debug_assert!(player_index < self.num_players);
        &self.player_knowledge[player_index]
    }

    /// Get a mutable reference to a specific player's knowledge.
    pub fn player_mut(&mut self, player_index: usize) -> &mut PlayerKnowledgeState {
        debug_assert!(player_index < self.num_players);
        &mut self.player_knowledge[player_index]
    }

    /// Update all players' knowledge when a new card is drawn by one of the players.
    ///
    /// Players other than the drawer can see the card and have their empathy updated.
    /// The drawer cannot see the card but knows it is now in their hand.
    pub fn update_with_card_drawn(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        for i in 0..self.num_players {
            if i != player_index {
                self.player_knowledge[i].update_with_revealed_card(
                    card_position_in_starting_deck,
                    card_id,
                );
            }
        }
        self.player_knowledge[player_index].own_hand |= 1 << card_position_in_starting_deck;
    }

    /// Number of active players.
    pub fn num_players(&self) -> usize {
        self.num_players
    }
}
