use crate::engine::convention::convention_tech::DiscardTech;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::MAX_CLUE_TOKEN_COUNT;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_discard_tech;

/// Discard the leftmost (newest, slot 1) known trash card in the active player's hand.
///
/// A card is known trash when every identity consistent with the player's empathy is
/// either already played or unreachable because a prerequisite was fully discarded.
///
/// When multiple known trash cards exist, the one in the lowest-numbered slot (newest,
/// first in `Hand::cards()`) is chosen. Returns no action when the hand holds no known
/// trash — in that case `DiscardChop` is expected to fire instead.
pub struct DiscardKnownTrash;

impl DiscardTech for DiscardKnownTrash {
    fn discard_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_index = active_player_pov.active_player_index();
        let table_state = active_player_pov.table_state();
        if table_state.clue_token_bank.whole_clue_tokens_count() >= MAX_CLUE_TOKEN_COUNT {
            return vec![];
        }
        let hand = &table_state.hands[player_index];
        // Hand::cards() is newest-first (slot 1 → slot N); first match = leftmost.
        match hand
            .cards()
            .iter()
            .copied()
            .find(|&idx| active_player_pov.is_known_trash(idx))
        {
            Some(card_deck_index) => vec![GameAction::Discard {
                player_index,
                card_deck_index,
                turn: table_state.current_turn,
            }],
            None => vec![],
        }
    }

    fn matches_discard(
        &self,
        _player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        observer_pov.is_known_trash(card_deck_index)
    }

    fn discard_knowledge_updates(
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

impl_convention_tech_for_discard_tech!(DiscardKnownTrash);

#[cfg(test)]
mod tests {
    mod integration {
        use crate::engine::convention::convention_tech::ConventionTech;
        use crate::engine::convention::hgroup::tech::discard_known_trash::DiscardKnownTrash;
        use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
        use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
        use crate::engine::knowledge::team_knowledge::TeamKnowledge;
        use crate::game::action::game_action::GameAction;
        use crate::game::card::CardIdentityMask;
        use crate::game::deck::unit_test_constants::novariant_constants::{NoVarCards, R1_MASK};
        use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
            NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
        };

        /// Build a knowledge where every card in `hand_cards` is in `own_hand`, and
        /// cards listed in `trash_empathy` have their `inferred_identities` set to the
        /// provided mask (used to mark them as known trash when that mask is fully played).
        fn make_knowledge(hand_cards: &[u8], trash_empathy: &[(u8, u64)]) -> PlayerKnowledge {
            let mut k = PlayerKnowledge::new(0);
            for &idx in hand_cards {
                k.own_hand |= 1u64 << idx;
            }
            for &(idx, mask) in trash_empathy {
                k.inferred_identities[idx as usize] = Some(CardIdentityMask::from_bits(mask));
            }
            k
        }

        /// Play R1 to the stacks so that any card with empathy R1_MASK becomes known trash.
        fn play_r1(
            table_state: &mut crate::game::state::table_state::TableState,
            static_data: &crate::game::static_game_data::StaticGameData,
        ) {
            table_state.update_with_draw_action(0);
            table_state.update_with_play_action_of_specific_card(
                0,
                NoVarCards::R1.as_variant_card_id(),
                static_data,
            );
        }

        #[test]
        fn returns_no_action_when_no_known_trash_in_hand() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            table_state.current_turn = 2;
            for &idx in &[10u8, 20, 30] {
                table_state.update_with_draw_action(idx);
            }
            // No empathy narrowing → no card is known trash.
            let knowledge = make_knowledge(&[10, 20, 30], &[]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            assert!(DiscardKnownTrash.game_actions(&pov).is_empty());
        }

        #[test]
        fn discards_known_trash_card_when_present() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            table_state.clue_token_bank.set_half_tokens(14); // 7 whole tokens (not max)
            table_state.current_turn = 3;
            // Draw hand cards: oldest→newest = [10, 20].
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            // Card 10 (oldest, slot 2) is known trash (empathy = R1, already played).
            // Card 20 (newest, slot 1) has no empathy → not trash.
            let knowledge = make_knowledge(&[10, 20], &[(10, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let actions = DiscardKnownTrash.game_actions(&pov);
            assert_eq!(
                actions,
                vec![GameAction::Discard {
                    player_index: 0,
                    card_deck_index: 10,
                    turn: 3,
                }]
            );
        }

        #[test]
        fn picks_leftmost_when_multiple_known_trash_cards() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            table_state.clue_token_bank.set_half_tokens(14); // 7 whole tokens (not max)
            table_state.current_turn = 4;
            // Draw hand: oldest→newest = [10, 20, 30].
            for &idx in &[10u8, 20, 30] {
                table_state.update_with_draw_action(idx);
            }
            // Cards 10 (slot 3) and 20 (slot 2) are known trash; 30 (slot 1) is not.
            // Leftmost trash = slot 2 = card 20.
            let knowledge = make_knowledge(&[10, 20, 30], &[(10, R1_MASK), (20, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let actions = DiscardKnownTrash.game_actions(&pov);
            assert_eq!(
                actions,
                vec![GameAction::Discard {
                    player_index: 0,
                    card_deck_index: 20,
                    turn: 4,
                }]
            );
        }

        #[test]
        fn picks_leftmost_when_all_cards_are_known_trash() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            table_state.clue_token_bank.set_half_tokens(14); // 7 whole tokens (not max)
            table_state.current_turn = 5;
            // Hand: oldest→newest = [10, 20]. Both trash.
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = make_knowledge(&[10, 20], &[(10, R1_MASK), (20, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            // Newest = card 20 (slot 1 = leftmost).
            let actions = DiscardKnownTrash.game_actions(&pov);
            assert_eq!(
                actions,
                vec![GameAction::Discard {
                    player_index: 0,
                    card_deck_index: 20,
                    turn: 5,
                }]
            );
        }

        #[test]
        fn returns_no_action_when_clue_tokens_at_max() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            // Clue tokens at max (16 half-tokens = 8 whole)
            table_state.current_turn = 3;
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = make_knowledge(&[10, 20], &[(10, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            assert!(DiscardKnownTrash.game_actions(&pov).is_empty());
        }

        #[test]
        fn matches_action_returns_true_for_known_trash_discard() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = make_knowledge(&[10, 20], &[(10, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let action = GameAction::Discard {
                player_index: 0,
                card_deck_index: 10,
                turn: 6,
            };
            assert!(DiscardKnownTrash.matches_action(&action, &[], &pov));
        }

        #[test]
        fn matches_action_returns_false_for_non_trash_discard() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            play_r1(&mut table_state, &static_data);
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            // Card 20 has no narrowed empathy → not known trash.
            let knowledge = make_knowledge(&[10, 20], &[(10, R1_MASK)]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let action = GameAction::Discard {
                player_index: 0,
                card_deck_index: 20,
                turn: 6,
            };
            assert!(!DiscardKnownTrash.matches_action(&action, &[], &pov));
        }
    }
}
