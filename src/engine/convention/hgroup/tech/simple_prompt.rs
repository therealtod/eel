use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue whose focus card is exactly 1 step away from playable, where the connecting card
/// is already touched (clued) in a teammate's hand but its identity is still unknown to that
/// teammate — prompting them to play it.
///
/// The prompted player will assume that the **leftmost** of their touched cards is the connecting
/// card and play it. Therefore this tech is only valid when one of the following holds:
///
/// - The leftmost touched card in the prompted player's hand **is** the connecting card, or
/// - Every touched card from the leftmost up to and including the actual connecting card is
///   currently playable on the stacks (a "prompt chain": the player plays each one in turn,
///   eventually reaching the connecting card).
/// See: https://hanabi.github.io/level-1#the-prompt and https://hanabi.github.io/beginner/prompt
pub struct SimplePrompt;

impl SimplePrompt {
    /// Validates whether `connecting_id` can be prompted in `target_player`'s hand.
    ///
    /// Validity requires:
    /// 1. The card is touched and its identity is unknown to its holder.
    /// 2. All touched cards in that player's hand that are to the left of (and including) the
    ///    connecting card are playable — so the player will naturally play through to it.
    fn is_valid_prompt_situation(
        connecting_id: VariantCardId,
        target_player: PlayerIndex,
        pov: &dyn PlayerPOV,
    ) -> bool {
        let hand = pov.table_state().hands[target_player].cards();

        let touched: Vec<CardDeckIndex> = hand
            .iter()
            .copied()
            .filter(|&idx| pov.is_touched(idx))
            .collect();

        let pos = touched.iter().position(|&idx| {
            pov.card_identity(idx) == Some(connecting_id) && !pov.is_identity_known_to_holder(idx)
        });

        match pos {
            None => false,
            Some(p) => touched[..=p].iter().all(|&idx| pov.is_playable(idx)),
        }
    }
}

impl ClueTech for SimplePrompt {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                clues_for_player_with_focus(target, pov)
                    .into_iter()
                    .filter_map(move |(action, focus_idx)| {
                        let card_id = pov.card_identity(focus_idx)?;
                        if pov.away_value(card_id) != Some(1) {
                            return None;
                        }
                        let connecting_id = card_id - 1;
                        if (0..num_players)
                            .filter(|&p| p != active)
                            .any(|p| Self::is_valid_prompt_situation(connecting_id, p, pov))
                        {
                            Some(action)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }

    fn matches_clue(
        &self,
        target_player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        let Some(game_state_snapshot) = history.get(turn) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());
        get_clue_focus(target_player_index, touched, &giver_pov)
            .and_then(|focus| giver_pov.card_identity(focus))
            .filter(|&card_id| giver_pov.away_value(card_id) == Some(1))
            .is_some_and(|card_id| {
                let connecting_id = card_id - 1;
                (0..giver_pov.static_data().number_of_players as usize)
                    .filter(|&p| p != giver_pov.active_player_index())
                    .any(|p| Self::is_valid_prompt_situation(connecting_id, p, &giver_pov))
            })
    }

    fn clue_knowledge_updates(
        &self,
        clue_receiver_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate> {
        let Some(snap) = history.get(turn) else {
            return vec![];
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let current = observer_pov.active_player_index();
        let static_data = giver_pov.static_data();
        let total_ids =
            static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;

        // ── Case 1: current player is the prompted player ─────────────────────
        if current != clue_receiver_index {
            let focus = get_clue_focus(clue_receiver_index, &touched, &giver_pov);
            if let Some(focus) = focus {
                let focus_id = match giver_pov.card_identity(focus) {
                    Some(id) if giver_pov.away_value(id) == Some(1) => id,
                    _ => return vec![],
                };
                let connecting_id = focus_id - 1;
                let hand = giver_pov.table_state().hands[current].cards();
                let touched_in_hand: Vec<CardDeckIndex> = hand
                    .iter()
                    .copied()
                    .filter(|&idx| giver_pov.is_touched(idx))
                    .collect();
                let pos = touched_in_hand.iter().position(|&idx| {
                    giver_pov.card_identity(idx) == Some(connecting_id)
                        && !giver_pov.is_identity_known_to_holder(idx)
                });
                if let Some(p) = pos {
                    if touched_in_hand[..=p]
                        .iter()
                        .all(|&idx| giver_pov.is_playable(idx))
                    {
                        if let Some(card) = touched_in_hand.first().copied() {
                            return vec![KnowledgeUpdate::NarrowPossibilities {
                                card_deck_index: card,
                                mask: 1 << connecting_id,
                            }];
                        }
                    }
                }
            }
            return vec![];
        }

        // ── Case 2: clue receiver ─────────────────────────────────────────────
        let clue_mask = giver_pov
            .static_data()
            .variant
            .empathy_for_clue(clue)
            .as_bits();
        if let Some(focus) = get_clue_focus(clue_receiver_index, &touched, &giver_pov) {
            let num_players = giver_pov.static_data().number_of_players as usize;
            let giver_idx = giver_pov.active_player_index();
            let mask: u64 = (0..total_ids)
                .filter(|&id| {
                    (1u64 << id) & clue_mask != 0
                        && giver_pov.away_value(id) == Some(1)
                        && (0..num_players)
                            .filter(|&p| p != giver_idx)
                            .any(|p| Self::is_valid_prompt_situation(id - 1, p, &giver_pov))
                })
                .fold(0u64, |acc, id| acc | (1 << id));
            if mask != 0 {
                return vec![KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index: focus,
                    mask,
                }];
            }
        }

        vec![]
    }
}

impl HGroupClueTech for SimplePrompt {}
impl PlayClueTech for SimplePrompt {}
impl_convention_tech_for_hgroup_clue_tech!(SimplePrompt, priority::PROMPT);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    /// Actor (player 0) knows the identities of all cards listed.
    /// `team_knowledge` is a fresh `TeamKnowledge` — sufficient for tests that don't need
    /// `is_identity_known_to_holder` to return `true`.
    fn setup(
        cards: &[(usize, u8, u64)],   // (player_index, deck_index, empathy_mask)
        touched: &[u8],               // deck indexes that are clue-touched
        stacks_played: &[NoVarCards], // cards already played onto stacks
    ) -> (
        crate::game::state::table_state::TableState,
        PlayerKnowledge,
        TeamKnowledge,
        crate::game::static_game_data::StaticGameData,
    ) {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        for &card in stacks_played {
            table_state.update_with_play_action_of_specific_card(
                0,
                card.as_variant_card_id(),
                &static_data,
            );
        }

        for &(player, deck_idx, _) in cards {
            table_state.active_player_index = player;
            table_state.update_with_draw_action(deck_idx as u8);
        }
        table_state.active_player_index = 0;

        for &idx in touched {
            table_state.clue_touched_cards |= 1u64 << idx;
        }

        let mut knowledge = PlayerKnowledge::new(0);
        for &(_, deck_idx, mask) in cards {
            knowledge.inferred_identities[deck_idx as usize] =
                Some(CardIdentityMask::from_bits(mask));
            knowledge.visible_cards |= 1u64 << deck_idx;
        }

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        for &(_, deck_idx, mask) in cards {
            team_knowledge.player_mut(0).inferred_identities[deck_idx as usize] =
                Some(CardIdentityMask::from_bits(mask));
            team_knowledge.player_mut(0).visible_cards |= 1u64 << deck_idx;
        }
        (table_state, knowledge, team_knowledge, static_data)
    }

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn returns_empty_when_connecting_card_is_not_touched() {
        // Player 1 has R2 (untouched). Player 2 has R3 (focus, 1-away). No prompt possible.
        let (table_state, knowledge, team_knowledge, static_data) = setup(
            &[(1, 10, R2_MASK), (2, 20, R3_MASK)],
            &[], // nothing touched
            &[R1],
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(SimplePrompt.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_connecting_card_identity_is_known_to_holder() {
        // Player 1 has R2 touched, but their identity IS known to them (visible_cards set).
        // This is delayed_play_clue territory, not a prompt.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R2
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R3
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= 1u64 << 10;

        let mut knowledge = PlayerKnowledge::new(0);
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(R2_MASK));
        knowledge.visible_cards |= 1u64 << 10;
        knowledge.inferred_identities[20] = Some(CardIdentityMask::from_bits(R3_MASK));
        knowledge.visible_cards |= 1u64 << 20;

        // Mark R2 (card 10) as known to its holder (player 1) in team_knowledge.
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= 1u64 << 10;
        team_knowledge.player_mut(1).visible_cards |= 1u64 << 10;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(SimplePrompt.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_focus_is_directly_playable() {
        // R1 is directly playable (away=0) — not a prompt target.
        let (table_state, knowledge, team_knowledge, static_data) =
            setup(&[(1, 10, R1_MASK)], &[], &[]);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(SimplePrompt.game_actions(&pov).is_empty());
    }

    #[test]
    fn generates_clue_when_connecting_card_is_leftmost_touched_and_unknown() {
        // Stack: R1 played. Player 1 has R2 (touched, unknown to holder) in slot 1 (leftmost).
        // Player 2 has R3 (focus, 1-away). Valid prompt.
        let (table_state, knowledge, team_knowledge, static_data) =
            setup(&[(1, 10, R2_MASK), (2, 20, R3_MASK)], &[10], &[R1]);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let actions = SimplePrompt.game_actions(&pov);
        assert!(actions.iter().any(|a| matches!(
            a,
            GameAction::Clue {
                player_index: 2,
                ..
            }
        )));
    }

    #[test]
    fn generates_clue_when_prompt_chain_all_left_cards_playable() {
        // Stacks: R1, R2, R3 played. Player 1 hand (oldest→newest = slot5→slot1):
        //   card 10 = R4 (slot 2), card 11 = Y1 (slot 1 = leftmost).
        // Both touched and unknown to holder. Leftmost = Y1 (playable). R4 is connecting card
        // for R5 (also playable, away=0). Player 2 has R5 (focus, 1-away).
        // Chain: Y1 (leftmost, playable) → R4 (connecting, playable) → valid prompt.
        let (table_state, knowledge, team_knowledge, static_data) = setup(
            &[(1, 10, R4_MASK), (1, 11, Y1_MASK), (2, 20, R5_MASK)],
            &[10, 11],
            &[R1, R2, R3],
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let actions = SimplePrompt.game_actions(&pov);
        assert!(actions.iter().any(|a| matches!(
            a,
            GameAction::Clue {
                player_index: 2,
                ..
            }
        )));
    }

    #[test]
    fn returns_empty_when_leftmost_touched_card_is_not_playable_and_not_the_connecting_card() {
        // Same as above but leftmost = Y2 (not playable) instead of Y1.
        // Chain: Y2 (leftmost, NOT playable) → invalid prompt.
        let (table_state, knowledge, team_knowledge, static_data) = setup(
            &[(1, 10, R4_MASK), (1, 11, Y2_MASK), (2, 20, R5_MASK)],
            &[10, 11],
            &[R1, R2, R3],
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(SimplePrompt.game_actions(&pov).is_empty());
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_for_valid_prompt_clue() {
        let (table_state, knowledge, team_knowledge, static_data) =
            setup(&[(1, 10, R2_MASK), (2, 20, R3_MASK)], &[10], &[R1]);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let clue = GameAction::Clue {
            player_index: 2,
            touched_card_deck_indexes: smallvec::smallvec![20],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 0,
        };
        assert!(SimplePrompt.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_directly_playable() {
        let (table_state, knowledge, team_knowledge, static_data) =
            setup(&[(1, 10, R1_MASK)], &[], &[]);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Rank,
                clue_value: 1,
            },
            turn: 0,
        };
        assert!(!SimplePrompt.matches_action(&clue, &[], &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(!SimplePrompt.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 0,
            },
            &[],
            &pov
        ));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    /// Clue receiver (player 0) has R3 as focus (1-away). Player 1 has R2 touched and unknown.
    /// knowledge_updates should narrow the focus card to all 1-away IDs with a valid prompted player.
    #[test]
    fn knowledge_updates_receiver_narrows_focus_to_1_away_ids_with_prompted_player() {
        // Stack: R1 played. Player 0 (receiver) has R3 (card 10, touched, focus).
        // Player 1 has R2 (card 20, touched, unknown to holder) — valid prompt.
        let (table_state, knowledge, team_knowledge, static_data) =
            setup(&[(0, 10, R3_MASK), (1, 20, R2_MASK)], &[10, 20], &[R1]);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let updates = SimplePrompt.knowledge_updates(
            &GameAction::Clue {
                player_index: 0,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 0,
            },
            &[snapshot],
            &pov,
        );

        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates[0]
        {
            assert_eq!(*card_deck_index, 10);
            assert_ne!(mask & R3_MASK, 0, "R3 should be in the mask");
            assert_eq!(
                mask & R1_MASK,
                0,
                "R1 (directly playable) should not be in the mask"
            );
            assert_eq!(
                mask & R2_MASK,
                0,
                "R2 (away=0 after R1 played) should not be in the mask"
            );
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    /// Clue receiver gets a NarrowPossibilities update when given a play clue on a 1-away card.
    #[test]
    fn knowledge_updates_prompted_player_narrows_leftmost_touched_to_connecting_card() {
        let (table_state, knowledge, mut team_knowledge, static_data) =
            setup(&[(0, 10, R3_MASK), (1, 20, R2_MASK)], &[10, 20], &[R1]);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 20;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = SimplePrompt.knowledge_updates(
            &GameAction::Clue {
                player_index: 0,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 0,
            },
            &[snapshot],
            &pov,
        );

        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates[0]
        {
            assert_eq!(*card_deck_index, 10);
            assert_ne!(mask & R3_MASK, 0, "R3 should be in the mask");
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    /// When the receiver has no touched cards, knowledge_updates returns empty.
    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        let (table_state, knowledge, team_knowledge, static_data) = setup(
            &[(0, 10, R3_MASK), (1, 20, R2_MASK)],
            &[], // nothing touched
            &[R1],
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        assert!(
            SimplePrompt
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 0
                    },
                    &[],
                    &pov
                )
                .is_empty()
        );
    }
}
