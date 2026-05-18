use crate::engine::convention::convention_tech::PlayTech;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_play_tech;

/// Play a card that carries a `Signal::Play` and is not clue-touched.
///
/// Blind plays arise from finesses and other convention techs that attach a play signal to an
/// untouched card. The signal carries the committed identity directly; downstream knowledge
/// updates (e.g. promoting other observers' interpretations) are handled via pending
/// interpretations, not from this tech.
pub struct BlindPlay;

impl PlayTech for BlindPlay {
    fn play_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_index = pov.active_player_index();
        let knowledge = pov.team_knowledge().player(player_index);
        let mut actions = vec![];
        let mut hand_mask = knowledge.own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            let has_play_signal = knowledge.signals[card_deck_index as usize]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }));
            if has_play_signal && !pov.is_touched(card_deck_index) {
                actions.push(GameAction::Play {
                    player_index,
                    card_deck_index,
                    turn: pov.table_state().current_turn,
                });
            }
            hand_mask &= !(1u64 << card_deck_index);
        }
        actions
    }

    fn matches_play(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        let knowledge = observer_pov.team_knowledge().player(player_index);
        let has_play_signal = knowledge.signals[card as usize]
            .iter()
            .any(|s| matches!(s, Signal::Play { .. }));
        has_play_signal && !observer_pov.is_touched(card)
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

impl_convention_tech_for_play_tech!(BlindPlay);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    fn pov_with_signal(
        card_deck_index: CardDeckIndex,
        signal: Signal,
        touched: bool,
    ) -> (
        crate::game::state::table_state::TableState,
        PlayerKnowledge,
        TeamKnowledge,
        crate::game::static_game_data::StaticGameData,
    ) {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(card_deck_index);
        if touched {
            table_state.clue_touched_cards |= 1 << card_deck_index;
        }
        table_state.active_player_index = 0;

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.own_hand = 1 << card_deck_index;
        knowledge.inferred_identities[card_deck_index as usize] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        knowledge.signals[card_deck_index as usize].push(signal.clone());

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << card_deck_index;
        team_knowledge.player_mut(0).signals[card_deck_index as usize].push(signal);

        (table_state, knowledge, team_knowledge, static_data)
    }

    fn play_signal(card_deck_index: CardDeckIndex) -> Signal {
        Signal::Play {
            card_deck_index,
            committed_identity: R2.as_variant_card_id(),
        }
    }

    #[test]
    fn generates_play_action_for_untouched_card_with_play_signal() {
        let (table_state, knowledge, team_knowledge, static_data) =
            pov_with_signal(5, play_signal(5), false);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = BlindPlay.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0
            }]
        );
    }

    #[test]
    fn no_action_for_touched_card_with_play_signal() {
        let (table_state, knowledge, team_knowledge, static_data) =
            pov_with_signal(5, play_signal(5), true);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(BlindPlay.game_actions(&pov).is_empty());
    }

    #[test]
    fn no_action_for_untouched_card_without_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(5);
        table_state.active_player_index = 0;

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.own_hand = 1 << 5u64;
        knowledge.inferred_identities[5] = Some(CardIdentityMask::from_bits(R2_MASK));
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << 5u64;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(BlindPlay.game_actions(&pov).is_empty());
    }

    #[test]
    fn matches_play_true_for_untouched_card_with_play_signal() {
        let (table_state, knowledge, team_knowledge, static_data) =
            pov_with_signal(5, play_signal(5), false);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(BlindPlay.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0
            },
            &[],
            &pov
        ));
    }

    #[test]
    fn matches_play_false_for_touched_card_with_play_signal() {
        let (table_state, knowledge, team_knowledge, static_data) =
            pov_with_signal(5, play_signal(5), true);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!BlindPlay.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0
            },
            &[],
            &pov
        ));
    }

    #[test]
    fn matches_play_false_for_untouched_card_without_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(5);
        table_state.active_player_index = 0;

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.own_hand = 1 << 5u64;
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = 1 << 5u64;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(!BlindPlay.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0
            },
            &[],
            &pov
        ));
    }

    #[test]
    fn knowledge_updates_returns_empty() {
        let (table_state, knowledge, team_knowledge, static_data) =
            pov_with_signal(5, play_signal(5), false);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = BlindPlay.knowledge_updates(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0,
            },
            &[],
            &pov,
        );

        assert!(updates.is_empty());
    }
}
