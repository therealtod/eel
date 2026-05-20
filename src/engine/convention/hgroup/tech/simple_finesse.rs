use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus, get_finesse_position, has_on_finesse_position,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate, PendingTrigger};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue whose focus card is exactly 1 step away from playable, where the connecting card
/// sits on the finesse position (first unclued slot) of a teammate who plays before the target.
/// See https://hanabi.github.io/level-1#the-finesse and https://hanabi.github.io/beginner/finesse
///
/// **From the receiver's POV** the finesse is one of several possible interpretations of the
/// clue (it can also be a direct play, a critical save, etc). This tech contributes a single
/// provisional hypothesis pinning the focus to the 1-away identity, with a [`PendingTrigger`]
/// that confirms when the would-be finessed teammate blind-plays. The dispatcher composes
/// this with hypotheses from sibling techs (DelayedPlayClue, CriticalSave, …); the receiver's
/// effective focus mask is their union, and confirmation prunes the siblings.
pub struct SimpleFinesse;

impl SimpleFinesse {
    /// Core finesse detection: checks if giving a clue about `focus_card` to `target` constitutes
    /// a valid finesse. Returns true if:
    ///   1. `focus_card` is exactly 1-away from playable, AND
    ///   2. The prerequisite card (focus_card - 1) sits on the finesse position of a teammate
    ///      who plays before `target`.
    fn is_finesse_setup(focus_card: VariantCardId, target: usize, pov: &dyn PlayerPOV) -> bool {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;

        if pov.away_value(focus_card) != Some(1) {
            return false;
        }
        let prerequisite = focus_card - 1;
        if pov.is_gotten(prerequisite) {
            return false;
        }

        (0..num_players)
            .filter(|&p| p != active && p != target)
            .any(|p| {
                pov.static_data().plays_before(p, target, active)
                    && has_on_finesse_position(prerequisite, p, pov)
            })
    }
}

impl ClueTech for SimpleFinesse {
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
                        if Self::is_finesse_setup(card_id, target, pov) {
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
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
            return false;
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let Some(focus) = get_clue_focus(target_player_index, touched, &giver_pov) else {
            return false;
        };
        // Match if any focus identity consistent with the observer's empathy and the clue mask
        // would have constituted a finesse setup from the giver's POV. For non-receiver observers
        // the empathy collapses to a singleton (they see the focus); for the receiver it is wider
        // and the existential captures her ambiguity over her own card.
        let static_data = observer_pov.static_data();
        let total_ids =
            static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let candidates = observer_pov.inferred_identities(focus).as_bits() & clue_mask;
        let num_players = static_data.number_of_players as usize;
        (0..total_ids).any(|focus_id| {
            if (candidates & (1u64 << focus_id)) == 0 {
                return false;
            }
            if giver_pov.away_value(focus_id) != Some(1) {
                return false;
            }
            let connecting_id = focus_id - 1;
            (0..num_players)
                .filter(|&p| p != giver && p != target_player_index)
                .any(|p| {
                    static_data.plays_before(p, target_player_index, giver)
                        && has_on_finesse_position(connecting_id, p, &giver_pov)
                })
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
    ) -> Hypothesis {
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
            return Hypothesis::empty();
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let observer = observer_pov.player_index();

        let focus = match get_clue_focus(clue_receiver_index, touched, &giver_pov) {
            Some(f) => f,
            None => return Hypothesis::empty(),
        };

        // ── Receiver branch ────────────────────────────────────────────────────
        // The receiver contributes a single provisional hypothesis: the focus is the 1-away id.
        // Other interpretations (direct play, critical save) come from sibling techs and the
        // dispatcher unions all the hypotheses' focus masks. On Bob's blind-play the trigger
        // confirms (siblings are pruned, focus pinned to one_away_id); otherwise the trigger
        // rejects and this hypothesis is dropped (siblings remain).
        if observer == clue_receiver_index {
            let num_players = giver_pov.static_data().number_of_players as usize;
            let static_data = giver_pov.static_data();
            let total_ids = static_data.variant.number_of_suits as usize
                * static_data.variant.stacks_size as usize;
            let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();

            let finessed_player = (0..num_players)
                .filter(|&p| p != giver && p != clue_receiver_index)
                .find(|&p| {
                    static_data.plays_before(p, clue_receiver_index, giver)
                        && get_finesse_position(p, &giver_pov).is_some_and(|finesse_idx| {
                            giver_pov.is_playable(finesse_idx)
                                && giver_pov.card_identity(finesse_idx).is_some_and(
                                    |connecting_id| {
                                        let focus_id = connecting_id + 1;
                                        focus_id < total_ids && (1u64 << focus_id) & clue_mask != 0
                                    },
                                )
                        })
                });

            let Some(finessed_player) = finessed_player else {
                return Hypothesis::empty();
            };
            let Some(finesse_position) = get_finesse_position(finessed_player, &giver_pov) else {
                return Hypothesis::empty();
            };
            let Some(connecting_id) = giver_pov.card_identity(finesse_position) else {
                return Hypothesis::empty();
            };
            let one_away_id = connecting_id + 1;

            return Hypothesis::provisional(
                vec![KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index: focus,
                    mask: 1u64 << one_away_id,
                }],
                PendingTrigger::blind_play(finessed_player, finesse_position),
            );
        }

        // ── Third-party branch (finessed player + spectators) ─────────────────
        // Resolve the finessed player from the giver's POV (which sees all hands).
        let focus_id = match giver_pov.card_identity(focus) {
            Some(id) if giver_pov.away_value(id) == Some(1) => id,
            _ => return Hypothesis::empty(),
        };
        let connecting_id = focus_id - 1;

        let num_players = observer_pov.static_data().number_of_players as usize;
        let Some(finessed_player_index) = (0..num_players)
            .filter(|&p| p != clue_receiver_index && p != giver)
            .find(|&p| {
                observer_pov
                    .static_data()
                    .plays_before(p, clue_receiver_index, giver)
                    && has_on_finesse_position(connecting_id, p, &giver_pov)
            })
        else {
            return Hypothesis::empty();
        };

        let Some(finesse_position) = get_finesse_position(finessed_player_index, &giver_pov) else {
            return Hypothesis::empty();
        };

        // The finessed player learns their finesse slot is the connecting card and gets a
        // Signal::Play. The dispatcher's own_hand filter keeps this update only for the
        // finessed player; spectators see the connecting card directly via visible_cards.
        Hypothesis::unconditional(vec![
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index: finesse_position,
                mask: 1 << connecting_id,
            },
            KnowledgeUpdate::AddSignal {
                card_deck_index: finesse_position,
                signal: Signal::Play {
                    card_deck_index: finesse_position,
                    committed_identity: connecting_id,
                },
            },
        ])
    }
}

impl HGroupClueTech for SimpleFinesse {}
impl PlayClueTech for SimpleFinesse {}
impl_convention_tech_for_hgroup_clue_tech!(SimpleFinesse, priority::FINESSE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::knowledge_with_visible;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── knowledge_updates: finesse position player ────────────────────────────

    /// Alice (player 0) gives a clue to Cathy (player 2) whose focus is R3 (1-away).
    /// Bob (player 1) has R2 on his finesse position. Observer is Bob: he should receive
    /// an AddSignal::Play on his finesse position card (deck index 10).
    #[test]
    fn knowledge_updates_finesse_position_player_gets_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );

        // Bob (player 1) draws R2 — his finesse position.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        // Cathy (player 2) draws R3 — the clue focus.
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);

        // Alice (player 0) is the clue giver. Snapshot captures the pre-clue state.
        table_state.active_player_index = 0;
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 20;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());

        // Observer is Bob (player 1, the finessed player).
        table_state.active_player_index = 1;
        let knowledge = knowledge_with_visible(1, &[(20, R3_MASK)]);
        let pov =
            LightweightPlayerPOV::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = SimpleFinesse.knowledge_updates(
            &GameAction::Clue {
                player_index: 2,
                touched_card_deck_indexes: smallvec::smallvec![20],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 1,
            },
            &[snapshot],
            &pov,
        );

        // Expect a NarrowPossibilities + AddSignal::Play on Bob's finesse position (deck 10).
        assert_eq!(updates.immediate.len(), 2);
        assert!(updates.trigger.is_none());
        assert!(matches!(
            &updates.immediate[0],
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index: 10,
                ..
            }
        ));
        assert!(matches!(
            &updates.immediate[1],
            KnowledgeUpdate::AddSignal {
                card_deck_index: 10,
                signal: Signal::Play { .. }
            }
        ));
    }

    /// No finesse: player 2's focus card is directly playable (away=0), not 1-away.
    /// Player 1 should NOT get a play signal.
    #[test]
    fn knowledge_updates_no_signal_when_focus_is_directly_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // some card in player 1's hand
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R1 in player 2's hand (away=0, touched)
        table_state.clue_touched_cards |= 1 << 20;
        table_state.active_player_index = 1;

        let knowledge = knowledge_with_visible(1, &[(10, R1_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            SimpleFinesse
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1,
                    },
                    &[],
                    &pov
                )
                .is_empty()
        );
    }

    /// No finesse: player 1's finesse position card is NOT the connecting card for player 2's focus.
    #[test]
    fn knowledge_updates_no_signal_when_finesse_position_card_does_not_match() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // Y2 — not the connecting card for R3
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R3 (focus, 1-away)
        table_state.clue_touched_cards |= 1 << 20;
        table_state.active_player_index = 1;

        let knowledge = knowledge_with_visible(1, &[(10, Y2_MASK), (20, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            SimpleFinesse
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1,
                    },
                    &[],
                    &pov
                )
                .is_empty()
        );
    }

    /// No touched cards on the receiver → empty updates.
    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10); // R3, NOT touched
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20); // R2 on finesse position
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            SimpleFinesse
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1,
                    },
                    &[],
                    &pov
                )
                .is_empty()
        );
    }

    /// Receiver branch: Cathy's hypothesis pins the focus to the 1-away id (R3) and is
    /// provisional on Bob blind-playing his finesse slot.
    #[test]
    fn knowledge_updates_receiver_returns_provisional_hypothesis() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );

        // Bob (player 1) has R2 (deck 10) — finesse position.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        // Cathy (player 2) has R3 (deck 20) — focus.
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);

        // Alice (player 0) is the giver.
        table_state.active_player_index = 0;
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Alice's POV: she sees R2 in Bob's hand, R3 in Cathy's hand.
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 20;
        team_knowledge.player_mut(2).own_hand |= 1u64 << 20;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());

        // Observer is Cathy (the receiver).
        table_state.active_player_index = 2;
        let knowledge = knowledge_with_visible(2, &[(10, R2_MASK)]);
        let pov =
            LightweightPlayerPOV::new(2, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = SimpleFinesse.knowledge_updates(
            &GameAction::Clue {
                player_index: 2,
                touched_card_deck_indexes: smallvec::smallvec![20],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 1,
            },
            &[snapshot],
            &pov,
        );

        // Hypothesis: pin focus to R3, with a BlindPlay trigger on Bob.
        assert_eq!(updates.immediate.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates.immediate[0]
        {
            assert_eq!(*card_deck_index, 20);
            assert_eq!(*mask, R3_MASK, "hypothesis pins focus to R3");
        } else {
            panic!("expected NarrowPossibilities");
        }

        match updates.trigger {
            Some(PendingTrigger::BlindPlay {
                player,
                expected_card,
                ..
            }) => {
                assert_eq!(player, 1);
                assert_eq!(expected_card, 10);
            }
            _ => panic!("expected BlindPlay trigger"),
        }
    }
}
