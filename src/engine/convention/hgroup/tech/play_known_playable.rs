use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;

/// Trivial technique: play a card that is known to be playable from the active player's POV.
pub struct PlayKnownPlayable;

impl ConventionTech for PlayKnownPlayable {
    fn priority(&self) -> u8 {
        0
    }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_on_turn_index = player_on_turn_pov.player_on_turn_index();
        let mut actions: Vec<GameAction> = vec![];
        let mut mask = player_on_turn_pov.own_playable_cards();
        while mask != 0 {
            let card_deck_index = mask.trailing_zeros() as CardDeckIndex;
            actions.push(GameAction::Play {
                player_index: player_on_turn_index,
                card_deck_index,
            });
            mask &= !(1 << card_deck_index);
        }
        actions
    }

    /// Returns true if the given action is explained by this technique from the actor's POV.
    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Play { card_deck_index, .. } = action {
            actor_pov.is_playable(*card_deck_index)
        } else {
            false
        }
    }

    fn knowledge_updates(&self, _player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::player_pov::MockPlayerPOV;
    use crate::game::card::DeckCardsBitField;

    #[test]
    fn should_return_no_actions_when_no_cards_are_playable() {
        let mut player_pov = MockPlayerPOV::new();
        player_pov.expect_own_playable_cards().return_const(0 as DeckCardsBitField);
        player_pov.expect_player_on_turn_index().return_const(0usize);

        let actual = PlayKnownPlayable.game_actions(&player_pov);

        assert!(actual.is_empty());
    }

    #[test]
    fn should_return_one_action_per_playable_card() {
        let mut player_pov = MockPlayerPOV::new();
        // bits 0 and 2: deck indices 0 and 2 are playable
        player_pov.expect_own_playable_cards().return_const(0b101 as DeckCardsBitField);
        player_pov.expect_player_on_turn_index().return_const(1usize);

        let actual = PlayKnownPlayable.game_actions(&player_pov);

        let expected = vec![
            GameAction::Play { player_index: 1, card_deck_index: 0 },
            GameAction::Play { player_index: 1, card_deck_index: 2 },
        ];
        assert_eq!(expected, actual);
    }
}
