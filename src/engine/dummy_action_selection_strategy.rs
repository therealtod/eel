use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::action::game_action::GameAction::{Discard, Play};
use crate::game::MAX_CLUE_TOKEN_COUNT;

struct DummyActionSelectionStrategy {}

impl ActionSelectionStrategy for DummyActionSelectionStrategy {
    fn select_active_player_action(
        &self,
        player_pov: &dyn PlayerPOV,
        _convention_set: &dyn ConventionSet,
    ) -> GameAction {
        let player_on_turn_index = player_pov.player_on_turn_index();
        let clue_tokens = player_pov.table_state().clue_token_bank.whole_clue_tokens_count();
        if clue_tokens == MAX_CLUE_TOKEN_COUNT {
            Play { player_index: player_on_turn_index, card_deck_index: 0 }
        } else {
            Discard { player_index: player_on_turn_index, card_deck_index: 0 }
        }
    }
}
