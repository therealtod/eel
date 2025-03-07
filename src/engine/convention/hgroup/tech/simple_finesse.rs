use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus, get_finesse_position, has_pending_play_signal,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue whose focus card is exactly 1 step away from playable, where the connecting card
/// sits on the finesse position (first unclued slot) of a teammate who plays before the target.
/// See https://hanabi.github.io/level-1#the-finesse and https://hanabi.github.io/beginner/finesse
pub struct SimpleFinesse;

impl SimpleFinesse {
    /// Returns true if `earlier` takes their turn before `later` in the circular order starting
    /// after the active player.
    fn plays_before(earlier: usize, later: usize, pov: &dyn PlayerPOV) -> bool {
        let n = pov.static_data().number_of_players as usize;
        let active = pov.active_player_index();
        let dist = |p: usize| (p + n - active) % n;
        dist(earlier) < dist(later)
    }

    /// Returns true if `player`'s finesse position (first unclued card, newest) holds `card_id`.
    fn has_on_finesse_position(card_id: VariantCardId, player: usize, pov: &dyn PlayerPOV) -> bool {
        pov.table_state().hands[player]
            .cards()
            .iter()
            .find(|&&idx| !pov.is_touched(idx))
            .map(|&idx| pov.card_identity(idx) == Some(card_id))
            .unwrap_or(false)
    }

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
        (0..num_players)
            .filter(|&p| p != active && p != target)
            .any(|p| {
                Self::plays_before(p, target, pov)
                    && Self::has_on_finesse_position(prerequisite, p, pov)
                    && get_finesse_position(p, pov)
                        .map(|fp| !has_pending_play_signal(p, fp, pov))
                        .unwrap_or(false)
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
        let Some(game_state_snapshot) = history.get(turn) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());

        get_clue_focus(target_player_index, touched, &giver_pov)
            .and_then(|focus| giver_pov.card_identity(focus))
            .map(|card_id| Self::is_finesse_setup(card_id, target_player_index, &giver_pov))
            .unwrap_or(false)
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

        let focus = match get_clue_focus(clue_receiver_index, touched, &giver_pov) {
            Some(f) => f,
            None => return vec![],
        };
        let focus_id = match giver_pov.card_identity(focus) {
            Some(id) if giver_pov.away_value(id) == Some(1) => id,
            _ => return vec![],
        };
        let connecting_id = focus_id - 1;

        // Find the finessed player using the giver's POV: it has full visibility of all hands,
        // so `card_identity` returns the actual identity for every player's card, including
        // the observer's own cards which are invisible to themselves.
        let num_players = observer_pov.static_data().number_of_players as usize;
        let Some(finessed_player_index) = (0..num_players)
            .filter(|&p| p != clue_receiver_index && p != giver)
            .find(|&p| {
                Self::plays_before(p, clue_receiver_index, &giver_pov)
                    && Self::has_on_finesse_position(connecting_id, p, &giver_pov)
            })
        else {
            return vec![];
        };

        let Some(finesse_position) = get_finesse_position(finessed_player_index, observer_pov)
        else {
            return vec![];
        };

        vec![
            KnowledgeUpdate::AddSignal {
                card_deck_index: finesse_position,
                signal: Signal::Play {
                    card_deck_index: finesse_position,
                    knowledge_updates: vec![KnowledgeUpdate::NarrowPossibilities {
                        card_deck_index: focus,
                        mask: 1 << (connecting_id + 1),
                    }],
                },
            },
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index: finesse_position,
                mask: 1 << connecting_id,
            },
        ]
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
    use crate::game::card::Empathy;
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
            Some(Empathy::from_bits(R2_MASK).unwrap());
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(Empathy::from_bits(R3_MASK).unwrap());
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
                turn: 0,
            },
            &[snapshot],
            &pov,
        );

        assert!(matches!(
            &updates[0],
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
                        turn: 0,
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
                        turn: 0,
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
                        turn: 0,
                    },
                    &[],
                    &pov
                )
                .is_empty()
        );
    }
}
