use crate::engine::convention::convention_tech::PlayTech;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_play_tech;

/// Trivial technique: play a card that is known to be playable from the active player's POV.
///
/// Only fires for empathy-playable cards (all remaining possibilities are playable). Untouched
/// cards that carry a `Signal::Play` are BlindPlay territory and are excluded here.
pub struct PlayKnownPlayable;

impl PlayTech for PlayKnownPlayable {
    fn play_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_index = active_player_pov.active_player_index();
        let knowledge = active_player_pov.team_knowledge().player(player_index);
        let table_state = active_player_pov.table_state();
        let playable = table_state.playable_cards(active_player_pov.static_data());
        let mut actions = vec![];
        let mut hand_mask = knowledge.own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            let has_play_signal = knowledge.signals[card_deck_index as usize]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }));
            // Use global empathy from Deck (game-rule based) merged with inferred identities
            let combined = knowledge.combined_possible_identities(
                card_deck_index,
                table_state,
                &active_player_pov.static_data().variant,
            );
            let bits = combined.as_bits();
            let empathy_playable = bits != 0 && bits & playable == bits;
            if empathy_playable && !has_play_signal {
                actions.push(GameAction::Play {
                    player_index,
                    card_deck_index,
                    turn: active_player_pov.table_state().current_turn,
                });
            }
            hand_mask &= !(1u64 << card_deck_index);
        }
        actions
    }

    fn matches_play(
        &self,
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        let knowledge = observer_pov.team_knowledge().player(player_index);
        let has_play_signal = knowledge.signals[card_deck_index as usize]
            .iter()
            .any(|s| matches!(s, Signal::Play { .. }));
        let table_state = observer_pov.table_state();
        let playable = table_state.playable_cards(observer_pov.static_data());
        let combined = knowledge.combined_possible_identities(
            card_deck_index,
            table_state,
            &observer_pov.static_data().variant,
        );
        let bits = combined.as_bits();
        bits != 0 && bits & playable == bits && !has_play_signal
    }

    fn play_knowledge_updates(
        &self,
        _player_index: PlayerIndex,
        _card: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        _observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        Hypothesis::empty()
    }
}

impl_convention_tech_for_play_tech!(PlayKnownPlayable);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    #[test]
    fn no_actions_when_hand_is_empty() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(PlayKnownPlayable.game_actions(&pov).is_empty());
    }

    #[test]
    fn plays_empathy_playable_card() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.current_turn = 2; // Expected turn in action
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.own_hand = 1 << 10;
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(R1_MASK));

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(
            PlayKnownPlayable.game_actions(&pov),
            vec![GameAction::Play {
                player_index: 0,
                card_deck_index: 10,
                turn: 2,
            }]
        );
    }

    #[test]
    fn no_action_for_empathy_playable_card_with_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let signal = Signal::Play {
            card_deck_index: 10,
            committed_identity: 0,
        };

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.own_hand = 1 << 10;
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(R1_MASK));
        knowledge.signals[10].push(signal.clone());

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).signals[10].push(signal);

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(PlayKnownPlayable.game_actions(&pov).is_empty());
    }

    #[test]
    fn matches_play_false_for_empathy_playable_card_with_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let signal = Signal::Play {
            card_deck_index: 10,
            committed_identity: 0,
        };

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).signals[10].push(signal);

        let knowledge = PlayerKnowledge::new(0);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!PlayKnownPlayable.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 10,
                turn: 2,
            },
            &[],
            &pov
        ));
    }
}
