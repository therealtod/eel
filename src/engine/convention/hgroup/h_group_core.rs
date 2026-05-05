use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::{MAX_CLUE_VALUES_PER_TYPE, MAX_HAND_SIZE};
use smallvec::SmallVec;

/// Reconstruct the POV of `pov.player_on_turn_index()` using that player's own
/// `team_knowledge` entry.
///
/// In `matches_clue` and `clue_game_actions`, `pov.player_on_turn_index()` is the clue giver,
/// so this genuinely gives the giver's perspective. In `clue_knowledge_updates` the runtime
/// sets `player_on_turn_index` to the current observer before calling, so `giver_pov` yields
/// the *observer's* own team-knowledge view — useful for checking what the observer already
/// knows about their own hand or other players' cards.
pub fn giver_pov(pov: &dyn PlayerPOV) -> LightweightPlayerPOV<'_> {
    pov.as_player_pov(pov.player_on_turn_index())
}

/// Returns the deck indices of cards in `player_index`'s hand that are touched by `clue`.
pub fn touched_cards_for_clue(
    player_index: usize,
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
                .map(|id| (1u64 << id) & empathy_mask != 0)
                .unwrap_or(false)
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
/// 2. Otherwise, if any newly-touched cards exist, the focus is the leftmost (newest, slot 1) one.
/// 3. If all touched cards were already clued, the focus is the leftmost (newest, slot 1) touched card.
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

/// Returns all valid clues for `target_player_index` paired with their focus card index.
///
/// For every (clue_type, clue_value) combination that touches at least one fully-known card in
/// the target's hand and produces a defined focus, one `(GameAction::Clue, focus_index)` entry
/// is emitted. Callers filter the list on whatever focus-card condition they need.
pub fn clues_for_player_with_focus(
    target_player_index: usize,
    player_pov: &dyn PlayerPOV,
) -> Vec<(GameAction, CardDeckIndex)> {
    let table_state = player_pov.table_state();
    let static_data = player_pov.static_data();

    if table_state.clue_token_bank.whole_clue_tokens_count() == 0 {
        return Vec::new();
    }

    let hand_cards: Vec<_> = table_state.hands[target_player_index].cards().to_vec();
    let mut result = Vec::new();

    for clue_type in &static_data.variant.clue_types {
        for clue_value in 0..MAX_CLUE_VALUES_PER_TYPE {
            let Some(empathy) = static_data.variant.empathy_by_clue(*clue_type, clue_value) else {
                continue;
            };
            let empathy_bits = empathy.as_bits();

            let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = hand_cards
                .iter()
                .copied()
                .filter(|&idx| {
                    player_pov
                        .card_identity(idx)
                        .map(|id| (1u64 << id) & empathy_bits != 0)
                        .unwrap_or(false)
                })
                .collect();

            if touched.is_empty() {
                continue;
            }

            let clue = Clue {
                clue_type: *clue_type,
                clue_value: clue_value as u8,
            };

            if let Some(focus_idx) = get_clue_focus(target_player_index, &touched, player_pov) {
                result.push((
                    GameAction::Clue {
                        player_index: target_player_index,
                        touched_card_deck_indexes: touched,
                        clue,
                        turn: None,
                    },
                    focus_idx,
                ));
            }
        }
    }
    result
}

/// https://hanabi.github.io/beginner/minimum-clue-value-principle/
///
/// A clue is MCVP-compliant if at least one of the touched cards was not previously clued.
/// A clue where every touched card was already touched is a Tempo Clue and must be rejected.
pub fn is_minimal_clue_value_compliant(
    _clue: &Clue,
    _clue_receiver_player_index: &PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    debug_assert!(!touched_cards.is_empty(), "No touched cards");
    let already_touched = player_pov.table_state().clue_touched_cards;
    touched_cards
        .iter()
        .any(|&idx| already_touched & (1u64 << idx) == 0)
}

/// Bitmask of variant card IDs still needed to complete the stacks.
fn still_needed_cards_mask(
    table_state: &TableState,
    static_data: &StaticGameData,
) -> VariantCardsBitField {
    let variant = &static_data.variant;
    let stacks_size = variant.stacks_size as usize;
    let mut needed: VariantCardsBitField = 0;
    for suit in 0..variant.number_of_suits as usize {
        let stack_top = table_state.playing_stacks.stack_size(suit) as usize;
        for rank_idx in stack_top..stacks_size {
            let card_id = suit * stacks_size + rank_idx;
            let total = variant.card_copies_count_by_id[card_id];
            let discarded = table_state.discard_pile.copies_of(card_id as VariantCardId);
            if discarded < total {
                needed |= 1u64 << card_id;
            }
        }
    }
    needed
}

/// Bitmask of variant card IDs that are exactly known and already clued in non-receiver hands.
fn already_clued_ids_mask(
    receiver: PlayerIndex,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> VariantCardsBitField {
    let num_players = static_data.number_of_players as usize;
    let mut mask: VariantCardsBitField = 0;
    for p in 0..num_players {
        if p == receiver {
            continue;
        }
        for &idx in table_state.hands[p].cards() {
            if table_state.clue_touched_cards & (1u64 << idx) != 0 {
                if let Some(id) = table_state.deck.get_global_empathy(idx).known_card_id() {
                    mask |= 1u64 << id;
                }
            }
        }
    }
    mask
}

/// Counts cards in `touched` that violate the good-touch principle.
///
/// A card is a bad touch if:
/// - its empathy has no overlap with still-needed cards, OR
/// - its exact identity is already clued in another player's hand.
pub(crate) fn count_bad_touches(
    touched: &[CardDeckIndex],
    receiver: PlayerIndex,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> usize {
    let still_needed = still_needed_cards_mask(table_state, static_data);
    let already_clued = already_clued_ids_mask(receiver, table_state, static_data);
    touched
        .iter()
        .filter(|&&idx| {
            let bits = table_state.deck.get_global_empathy(idx).as_bits();
            bits & still_needed == 0 || bits & already_clued != 0
        })
        .count()
}

pub fn is_good_touch_principle_compliant(
    _clue: &Clue,
    clue_receiver_player_index: &PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    count_bad_touches(
        touched_cards,
        *clue_receiver_player_index,
        player_pov.table_state(),
        player_pov.static_data(),
    ) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge_state::{
        knowledge_for_hand, knowledge_with_visible,
    };
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── giver_pov ──────────────────────────────────────────────────────

    #[test]
    fn giver_pov_returns_pov_of_giver() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let g = giver_pov(&pov);
        assert_eq!(g.player_on_turn_index(), 0);
    }

    #[test]
    fn giver_pov_respects_different_turn_index() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 2;
        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let g = giver_pov(&pov);
        assert_eq!(g.player_on_turn_index(), 2);
    }

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

    // ── clues_for_player_with_focus ────────────────────────────────────────

    #[test]
    fn clues_for_player_returns_empty_when_no_card_identity_is_known() {
        // Player 0 does not know card 10's identity → no clue can be computed.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[]); // card 10 not visible
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(clues_for_player_with_focus(1, &pov).is_empty());
    }

    #[test]
    fn clues_for_player_returns_one_entry_per_clue_that_touches_known_card() {
        // Player 1 has R1 (card 10). Color-red and rank-1 clues both touch it.
        // Both produce focus = 10 (the only card = chop).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let result = clues_for_player_with_focus(1, &pov);

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, focus)| *focus == 10));
        assert!(result.iter().any(|(action, _)| matches!(action,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Color // color clue
        )));
        assert!(result.iter().any(|(action, _)| matches!(action,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank // rank clue
        )));
    }

    #[test]
    fn clues_for_player_focus_is_chop_when_chop_is_touched() {
        // Player 1 hand oldest→newest: [card 10 = R2 (chop), card 20 = R1].
        // Red clue touches both → focus = chop (10, R2).
        // Rank-1 clue touches only card 20 → focus = 20 (R1).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2, oldest
        table_state.update_with_draw_action(20); // R1, newest
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let result = clues_for_player_with_focus(1, &pov);

        let red_entry = result.iter().find(|(action, _)| {
            matches!(action,
                GameAction::Clue { clue, touched_card_deck_indexes, .. }
                if clue.clue_type == ClueType::Color && touched_card_deck_indexes.len() == 2
            )
        });
        assert_eq!(
            red_entry.map(|(_, f)| *f),
            Some(10),
            "red clue focus should be chop (R2)"
        );

        let rank1_entry = result.iter().find(|(action, _)| {
            matches!(action,
                GameAction::Clue { clue, touched_card_deck_indexes, .. }
                if clue.clue_type == ClueType::Rank && *touched_card_deck_indexes == [20u8].into()
            )
        });
        assert_eq!(
            rank1_entry.map(|(_, f)| *f),
            Some(20),
            "rank-1 clue focus should be R1"
        );
    }

    // ── is_minimal_clue_value_compliant ────────────────────────────────────

    #[test]
    fn mcvp_passes_when_at_least_one_touched_card_is_new() {
        use crate::game::clue::Clue;
        use smallvec::smallvec;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(20);
        // Card 10 was already touched; card 20 is new.
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Rank,
            clue_value: 1,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![10, 20];

        assert!(is_minimal_clue_value_compliant(&clue, &0, &touched, &pov));
    }

    #[test]
    fn mcvp_fails_when_all_touched_cards_were_already_clued() {
        use crate::game::clue::Clue;
        use smallvec::smallvec;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(20);
        // Both cards already touched — this is a tempo clue.
        table_state.clue_touched_cards |= (1 << 10) | (1 << 20);

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Rank,
            clue_value: 1,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![10, 20];

        assert!(!is_minimal_clue_value_compliant(&clue, &0, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_for_single_untouched_card() {
        use crate::game::clue::Clue;
        use smallvec::smallvec;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![10];

        assert!(is_minimal_clue_value_compliant(&clue, &0, &touched, &pov));
    }
}
