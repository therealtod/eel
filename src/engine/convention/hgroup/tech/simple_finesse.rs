use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{clues_for_player_with_focus, get_clue_focus_index};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::signal::Signal;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};

/// Give a clue whose focus card is exactly 1 step away from playable, where the connecting card
/// sits on the finesse position (first unclued slot) of a teammate who plays before the target.
pub struct SimpleFinesse;

impl SimpleFinesse {
    /// Returns true if `earlier` takes their turn before `later` in the circular order starting
    /// after the active player.
    fn plays_before(earlier: usize, later: usize, pov: &dyn PlayerPOV) -> bool {
        let n = pov.static_data().number_of_players as usize;
        let active = pov.player_on_turn_index();
        let dist = |p: usize| (p + n - active) % n;
        dist(earlier) < dist(later)
    }

    /// Returns true if `player`'s finesse position (first unclued card, newest) holds `card_id`.
    fn has_on_finesse_position(card_id: VariantCardId, player: usize, pov: &dyn PlayerPOV) -> bool {
        pov.table_state().hands[player]
            .cards()
            .iter()
            .rev() // newest first
            .find(|&&idx| !pov.is_touched(idx))
            .map(|&idx| pov.card_identity(idx) == Some(card_id))
            .unwrap_or(false)
    }
}

impl ConventionTech for SimpleFinesse {
    fn priority(&self) -> u8 {
        3
    }

    fn game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.player_on_turn_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                clues_for_player_with_focus(target, pov)
                    .into_iter()
                    .filter_map(move |(action, focus_idx)| {
                        let card_id = pov.card_identity(focus_idx)?;
                        let away = pov.away_value(card_id);
                        if away != 1 {
                            return None;
                        }
                        let prerequisite = card_id - 1;
                        let finessed = (0..num_players)
                            .filter(|&p| p != active && p != target)
                            .any(|p| {
                                Self::plays_before(p, target, pov)
                                    && Self::has_on_finesse_position(prerequisite, p, pov)
                            });
                        if finessed { Some(action) } else { None }
                    })
            })
            .collect()
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Clue { player_index, touched_card_deck_indexes, .. } = action {
            let active = actor_pov.player_on_turn_index();
            let num_players = actor_pov.static_data().number_of_players as usize;
            let target = *player_index;

            get_clue_focus_index(target, touched_card_deck_indexes, actor_pov)
                .and_then(|focus| actor_pov.card_identity(focus))
                .map(|card_id| {
                    if actor_pov.away_value(card_id) != 1 {
                        return false;
                    }
                    let prerequisite = card_id - 1;
                    (0..num_players)
                        .filter(|&p| p != active && p != target)
                        .any(|p| {
                            Self::plays_before(p, target, actor_pov)
                                && Self::has_on_finesse_position(prerequisite, p, actor_pov)
                        })
                })
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn knowledge_updates(&self, pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        let current = pov.player_on_turn_index();
        let num_players = pov.static_data().number_of_players as usize;
        let total_ids = pov.static_data().variant.number_of_suits as usize
            * pov.static_data().variant.stacks_size as usize;

        // ── Case 1: current player is the finesse position player ─────────────
        // Check if any valid finesse clue exists where `current`'s finesse position
        // card is the connecting card. If so, signal them to blind-play it.
        let finesse_card = pov.table_state().hands[current]
            .cards()
            .iter()
            .rev()
            .copied()
            .find(|&idx| !pov.is_touched(idx));

        if let Some(fp_card) = finesse_card {
            if let Some(connecting_id) = pov.card_identity(fp_card) {
                // Is there a clue receiver (someone who plays after `current`) whose
                // focus is exactly 1-away and needs `connecting_id` as the bridge?
                let is_finessed = (0..num_players)
                    .filter(|&p| p != current && Self::plays_before(current, p, pov))
                    .any(|receiver| {
                        let recv_touched: Vec<CardDeckIndex> = pov.table_state().hands[receiver]
                            .cards()
                            .iter()
                            .copied()
                            .filter(|&idx| pov.is_touched(idx))
                            .collect();
                        get_clue_focus_index(receiver, &recv_touched, pov)
                            .and_then(|f| pov.card_identity(f))
                            .map(|focus_id| {
                                pov.away_value(focus_id) == 1 && focus_id - 1 == connecting_id
                            })
                            .unwrap_or(false)
                    });

                if is_finessed {
                    return vec![KnowledgeUpdate::AddSignal {
                        card_deck_index: fp_card,
                        signal: Signal::Play { slot_index: 1, turn: current },
                    }];
                }
            }
        }

        // ── Case 2: current player is the clue receiver ───────────────────────
        // They received a clue. They interpret it as a play clue (direct or delayed),
        // not knowing it's a finesse. Narrow focus to 1-away IDs where a valid finesse exists.
        let touched: Vec<CardDeckIndex> = pov.table_state().hands[current]
            .cards()
            .iter()
            .copied()
            .filter(|&idx| pov.is_touched(idx))
            .collect();
        if let Some(focus) = get_clue_focus_index(current, &touched, pov) {
            let mask: u64 = (0..total_ids)
                .filter(|&id| {
                    if pov.away_value(id) != 1 {
                        return false;
                    }
                    let prerequisite = id - 1;
                    (0..num_players)
                        .filter(|&p| p != current)
                        .any(|p| Self::has_on_finesse_position(prerequisite, p, pov))
                })
                .fold(0u64, |acc, id| acc | (1 << id));
            if mask != 0 {
                return vec![KnowledgeUpdate::NarrowPossibilities { card_deck_index: focus, mask }];
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
    use crate::engine::knowledge::player_pov_view::PlayerPOVView;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::engine::signal::Signal;
    use crate::game::deck::unit_test_constant::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constant::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        initial_five_players_table_state, NOVAR_5_PLAYERS_STATIC_GAME_DATA,
    };

    fn knowledge_with_visible(player_index: usize, visible: &[(u8, u64)]) -> PlayerKnowledgeState {
        let mut k = PlayerKnowledgeState::new(player_index);
        for &(idx, mask) in visible {
            k.empathy[idx as usize] = mask;
            k.visible_cards |= 1 << idx;
        }
        k
    }

    // ── knowledge_updates: finesse position player ────────────────────────────

    /// Player 1 (finesse position player) has R2 on their finesse position (unclued, newest).
    /// Player 2 (clue receiver) has R3 as focus (1-away, touched).
    /// POV is player 1 → should get AddSignal::Play on their finesse position card.
    #[test]
    fn knowledge_updates_finesse_position_player_gets_play_signal() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2 in player 1's hand (finesse position)
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // R3 in player 2's hand (focus, touched)
        table_state.clue_touched_cards |= 1 << 20;
        table_state.player_on_turn_index = 1; // POV is player 1

        let knowledge = knowledge_with_visible(1, &[(10, R2_MASK), (20, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = SimpleFinesse.knowledge_updates(&pov);

        assert_eq!(updates.len(), 1);
        assert!(matches!(
            &updates[0],
            KnowledgeUpdate::AddSignal { card_deck_index: 10, signal: Signal::Play { slot_index: 1, .. } }
        ));
    }

    /// No finesse: player 2's focus card is directly playable (away=0), not 1-away.
    /// Player 1 should NOT get a play signal.
    #[test]
    fn knowledge_updates_no_signal_when_focus_is_directly_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // some card in player 1's hand
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // R1 in player 2's hand (away=0, touched)
        table_state.clue_touched_cards |= 1 << 20;
        table_state.player_on_turn_index = 1;

        let knowledge = knowledge_with_visible(1, &[(10, R1_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(SimpleFinesse.knowledge_updates(&pov).is_empty());
    }

    /// No finesse: player 1's finesse position card is NOT the connecting card for player 2's focus.
    #[test]
    fn knowledge_updates_no_signal_when_finesse_position_card_does_not_match() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // Y2 — not the connecting card for R3
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // R3 (focus, 1-away)
        table_state.clue_touched_cards |= 1 << 20;
        table_state.player_on_turn_index = 1;

        let knowledge = knowledge_with_visible(1, &[(10, Y2_MASK), (20, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(1, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(SimpleFinesse.knowledge_updates(&pov).is_empty());
    }

    // ── knowledge_updates: clue receiver ─────────────────────────────────────

    /// Player 0 (clue receiver) has R3 as focus (1-away, touched).
    /// Player 1 has R2 on their finesse position (unclued, newest) and plays before player 0.
    /// POV is player 0 → focus should be narrowed to 1-away IDs with a valid finesse.
    #[test]
    fn knowledge_updates_receiver_narrows_focus_to_1_away_ids_with_finesse() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        // Player 0 draws R3 (focus, touched)
        table_state.player_on_turn_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;
        // Player 1 draws R2 (finesse position, unclued)
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(20);
        table_state.player_on_turn_index = 0; // POV is player 0

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = SimpleFinesse.knowledge_updates(&pov);

        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities { card_deck_index, mask } = &updates[0] {
            assert_eq!(*card_deck_index, 10);
            assert!(mask & R3_MASK != 0, "R3 should be in the mask");
            assert!(mask & R1_MASK == 0, "R1 (directly playable) should not be in the mask");
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    /// No touched cards on the receiver → empty updates.
    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 0;
        table_state.update_with_draw_action(10); // R3, NOT touched
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(20); // R2 on finesse position
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(SimpleFinesse.knowledge_updates(&pov).is_empty());
    }
}
