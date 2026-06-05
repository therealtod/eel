use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::MAX_HAND_SIZE;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use smallvec::SmallVec;

/// Returns the deck indices of cards in `player_index`'s hand that are touched by `clue`.
pub fn touched_cards_for_clue(
    player_index: PlayerIndex,
    clue: &Clue,
    player_pov: &dyn PlayerPOV,
) -> SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> {
    let empathy_mask = player_pov
        .static_data()
        .variant
        .empathy_for_clue(clue)
        .as_bits();
    player_pov.table_state().hands[player_index]
        .cards()
        .iter()
        .copied()
        .filter(|&idx| {
            player_pov
                .card_identity(idx)
                .is_some_and(|id| (1u64 << id) & empathy_mask != 0)
        })
        .collect()
}

/// Returns the chop card index for the given player: the oldest unclued card in their hand,
/// or `None` if every card has been clued. A hand always has at most one chop.
pub fn get_chop_index(player_index: usize, player_pov: &dyn PlayerPOV) -> Option<CardDeckIndex> {
    let hand = &player_pov.table_state().hands[player_index];
    hand.cards()
        .iter()
        .rev()
        .copied()
        .find(|&idx| !player_pov.is_touched(idx))
}

/// Returns the focus card index of a clue that touched `touched` in the given player's hand.
///
/// Focus rules (H-Group):
/// 1. If the chop is among the touched cards, the chop is the focus.
/// 2. Otherwise, if any newly touched cards exist, the focus is the leftmost (newest) one.
/// 3. If all touched cards were already clued, the focus is the leftmost (newest) touched card.
pub fn get_clue_focus(
    player_index: PlayerIndex,
    touched: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> Option<CardDeckIndex> {
    if let Some(chop) = get_chop_index(player_index, player_pov) {
        if touched.contains(&chop) {
            return Some(chop);
        }
    }

    // Leftmost = newest = first in cards() (which is ordered newest-first).
    let hand = &player_pov.table_state().hands[player_index];
    let leftmost_new = hand
        .cards()
        .iter()
        .copied()
        .find(|&idx| touched.contains(&idx) && !player_pov.is_touched(idx));

    leftmost_new.or_else(|| {
        // All touched cards already clued: focus is the leftmost touched card.
        hand.cards()
            .iter()
            .copied()
            .find(|&idx| touched.contains(&idx))
    })
}

pub fn get_finesse_position(
    player_index: PlayerIndex,
    player_pov: &dyn PlayerPOV,
) -> Option<CardDeckIndex> {
    let hand = &player_pov.table_state().hands[player_index];
    hand.cards()
        .iter()
        .copied()
        .find(|&idx| !player_pov.is_touched(idx))
}

/// Returns the deck index of the card that a prompt for `connecting_id` would cause
/// `target_player` to play, or `None` if no valid prompt situation exists.
///
/// A prompt assumes the holder plays the **leftmost** (newest) of their touched cards that
/// could be the connecting card. This is valid only when one of the following holds:
///
/// - Chain: every touched card from the leftmost up to and including the actual connecting card
///   is currently playable (the holder plays each in turn, eventually reaching the connector), or
/// - Skip: the first touched card whose holder empathy admits `connecting_id` IS the connecting
///   card — the holder skips over any preceding cards whose clue empathy excludes that identity.
///
/// `focus_idx` is excluded from the search: a self-prompt's focus is the card being newly clued,
/// and its fresh clue is exactly what disambiguates it from the connecting card.
pub fn prompt_connector_index(
    connecting_id: VariantCardId,
    target_player: PlayerIndex,
    focus_idx: CardDeckIndex,
    pov: &dyn PlayerPOV,
) -> Option<CardDeckIndex> {
    let hand = pov.table_state().hands[target_player].cards();

    let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = hand
        .iter()
        .copied()
        .filter(|&idx| idx != focus_idx && pov.is_touched(idx))
        .collect();

    let pos = touched.iter().position(|&idx| {
        pov.card_identity(idx) == Some(connecting_id) && !pov.is_identity_known_to_holder(idx)
    })?;

    // Chain: all cards from leftmost touched up to and including the connecting card are playable.
    if touched[..=pos].iter().all(|&idx| pov.is_playable(idx)) {
        return Some(touched[pos]);
    }

    // Skip: the first touched card whose holder empathy admits connecting_id must BE the connector.
    let holder_knowledge = pov.team_knowledge().player(target_player);
    for &idx in &touched[..=pos] {
        let holder_empathy = holder_knowledge.combined_possible_identities(
            idx,
            pov.table_state(),
            &pov.static_data().variant,
        );
        if holder_empathy.as_bits() & (1u64 << connecting_id) != 0 {
            return (pov.card_identity(idx) == Some(connecting_id)
                && !pov.is_identity_known_to_holder(idx))
            .then_some(idx);
        }
    }
    None
}

/// Returns true if `player_index` already has a pending `Signal::Play` on `card_deck_index`.
///
/// Use this in `clue_game_actions` to avoid generating a play clue or finesse for a card that
/// is already committed to being played (e.g., from an earlier finesse or prompt on this card).
pub fn has_pending_play_signal(
    player_index: PlayerIndex,
    card_deck_index: CardDeckIndex,
    pov: &dyn PlayerPOV,
) -> bool {
    pov.team_knowledge().player(player_index).signals[card_deck_index as usize]
        .iter()
        .any(|s| matches!(s, Signal::Play { .. }))
}

/// Returns true if `player`'s finesse position (first unclued card, newest) holds `card_id`.
pub fn has_on_finesse_position(
    card_id: VariantCardId,
    player_index: usize,
    observer_pov: &dyn PlayerPOV,
) -> bool {
    observer_pov.table_state().hands[player_index]
        .cards()
        .iter()
        .find(|&&idx| !observer_pov.is_touched(idx))
        .is_some_and(|&idx| observer_pov.card_identity(idx) == Some(card_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{knowledge_for_hand, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::deck::unit_test_constants::novariant_constants::{
        NoVarCards, R1_MASK, R2_MASK, R3_MASK,
    };
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── get_chop_index ──────────────────────────────────────────────────────

    #[test]
    fn chop_is_oldest_card_when_all_unclued() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_chop_index(0, &pov), Some(10));
    }

    #[test]
    fn chop_skips_clued_cards() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        table_state.clue_touched_cards |= 1 << 10;
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_chop_index(0, &pov), Some(20));
    }

    #[test]
    fn chop_is_none_when_all_cards_are_clued() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20] {
            table_state.update_with_draw_action(idx);
        }
        table_state.clue_touched_cards |= (1 << 10) | (1 << 20);
        let knowledge = knowledge_for_hand(&[10, 20]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_chop_index(0, &pov), None);
    }

    // ── get_clue_focus_index ────────────────────────────────────────────────

    #[test]
    fn focus_is_chop_when_chop_is_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Hand oldest→newest: [10, 20, 30]. Chop = 10.
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Clue touches chop (10) and an interior card (30).
        assert_eq!(get_clue_focus(0, &[10, 30], &pov), Some(10));
    }

    #[test]
    fn focus_is_newest_newly_touched_card_when_chop_not_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Hand oldest→newest: [10, 20, 30, 40, 50]. Chop = 10.
        for &idx in &[10u8, 20, 30, 40, 50] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Clue touches 20 and 50 (neither is chop). Newest touched = 50 (slot 1).
        assert_eq!(get_clue_focus(0, &[20, 50], &pov), Some(50));
    }

    #[test]
    fn focus_skips_already_clued_cards_when_finding_newest_new_touch() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Hand oldest→newest: [10, 20, 30, 40, 50]. Chop = 10.
        for &idx in &[10u8, 20, 30, 40, 50] {
            table_state.update_with_draw_action(idx);
        }
        // 50 was already touched by a previous clue.
        table_state.clue_touched_cards |= 1 << 50;
        let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Clue touches 20 and 50. 50 is already clued, so focus = 20 (newest new touch).
        assert_eq!(get_clue_focus(0, &[20, 50], &pov), Some(20));
    }

    #[test]
    fn focus_is_leftmost_touched_when_all_touched_cards_were_already_clued() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        // Mark all touched cards as already touched by a previous clue.
        table_state.clue_touched_cards |= (1 << 20) | (1 << 30);
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Chop (10) not touched; all touched already clued → focus = leftmost (newest) = 30.
        assert_eq!(get_clue_focus(0, &[20, 30], &pov), Some(30));
    }

    #[test]
    fn focus_skips_signal_touched_cards_when_finding_newest_new_touch() {
        // Hand oldest→newest: [10, 20, 30, 40, 50]. Chop = 10.
        // Card 50 carries a Play signal (signal-touched) but is NOT clue-touched.
        // Clue touches 20 and 50. 50 is signal-touched so it counts as "already touched",
        // making 20 the focus (newest new touch).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30, 40, 50] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let tk = team_knowledge.player_mut(0);
        tk.own_hand |= 1 << 50;
        tk.add_signal(
            50,
            Signal::Play {
                card_deck_index: 50,
                committed_identity: 0,
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_clue_focus(0, &[20, 50], &pov), Some(20));
    }

    #[test]
    fn focus_is_newest_new_touch_when_chop_is_signal_touched_but_not_clue_touched() {
        // Hand oldest→newest: [10, 20, 30]. Card 10 carries a Play signal (signal-touched)
        // but is NOT clue-touched, so chop becomes 20.
        // Clue touches 10 and 30. Since 10 is signal-touched (already touched),
        // focus = 30 (newest new touch).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let tk = team_knowledge.player_mut(0);
        tk.own_hand |= 1 << 10;
        tk.add_signal(
            10,
            Signal::Play {
                card_deck_index: 10,
                committed_identity: 0,
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_clue_focus(0, &[10, 30], &pov), Some(30));
    }

    #[test]
    fn chop_skips_signal_touched_cards() {
        // Hand oldest→newest: [10, 20, 30]. Card 10 has a Play signal (signal-touched).
        // Chop should be 20, not 10, because signal-touched cards are considered touched.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let tk = team_knowledge.player_mut(0);
        tk.own_hand |= 1 << 10;
        tk.add_signal(
            10,
            Signal::Play {
                card_deck_index: 10,
                committed_identity: 0,
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_chop_index(0, &pov), Some(20));
    }

    #[test]
    fn focus_is_signal_touched_card_when_all_other_touched_are_clue_touched() {
        // Hand oldest→newest: [10, 20, 30]. Chop = 10.
        // Card 30 is signal-touched (Play signal), card 20 is already clue-touched.
        // Clue touches 20 and 30. 20 is clue-touched, 30 is signal-touched — both
        // count as "already touched", so focus falls back to leftmost touched = 30.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        table_state.clue_touched_cards |= 1 << 20;
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let tk = team_knowledge.player_mut(0);
        tk.own_hand |= 1 << 30;
        tk.add_signal(
            30,
            Signal::Play {
                card_deck_index: 30,
                committed_identity: 0,
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(get_clue_focus(0, &[20, 30], &pov), Some(30));
    }

    // ── has_on_finesse_position ───────────────────────────────────────────

    #[test]
    fn returns_true_when_finesse_position_card_matches() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Player 0 draws R1 (card 10) then R2 (card 11). Newest = R2.
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        // Reveal card identities in deck.
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.deck.reveal_card(
            11,
            NoVarCards::R2.as_variant_card_id(),
            &static_data.variant,
        );

        // Observer knows both cards.
        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (11, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Finesse position = newest unclued card = R2 (card 11).
        assert!(has_on_finesse_position(
            NoVarCards::R2.as_variant_card_id(),
            0,
            &pov
        ));
    }

    #[test]
    fn returns_false_when_finesse_position_card_does_not_match() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.deck.reveal_card(
            11,
            NoVarCards::R2.as_variant_card_id(),
            &static_data.variant,
        );

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (11, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Finesse position = R2, not R3.
        assert!(!has_on_finesse_position(
            NoVarCards::R3.as_variant_card_id(),
            0,
            &pov
        ));
    }

    #[test]
    fn returns_false_when_all_cards_are_clued() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10); // R1
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        // Mark the only card as clued.
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // No unclued cards → no finesse position.
        assert!(!has_on_finesse_position(
            NoVarCards::R1.as_variant_card_id(),
            0,
            &pov
        ));
    }

    #[test]
    fn returns_false_when_hand_is_empty() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!has_on_finesse_position(
            NoVarCards::R1.as_variant_card_id(),
            0,
            &pov
        ));
    }

    #[test]
    fn finesse_position_is_newest_unclued_card() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Draw order: R1 (oldest), R2, R3 (newest).
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        table_state.update_with_draw_action(12); // R3
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.deck.reveal_card(
            11,
            NoVarCards::R2.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.deck.reveal_card(
            12,
            NoVarCards::R3.as_variant_card_id(),
            &static_data.variant,
        );
        // Clue R1 and R2, leaving R3 as the only unclued card.
        table_state.clue_touched_cards |= (1 << 10) | (1 << 11);

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (11, R2_MASK), (12, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Finesse position = R3 (newest unclued).
        assert!(has_on_finesse_position(
            NoVarCards::R3.as_variant_card_id(),
            0,
            &pov
        ));
        assert!(!has_on_finesse_position(
            NoVarCards::R2.as_variant_card_id(),
            0,
            &pov
        ));
    }
}
