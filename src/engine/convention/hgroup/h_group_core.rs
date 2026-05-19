use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::{MAX_CLUE_VALUES_PER_TYPE, MAX_HAND_SIZE};
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
            let empathy = static_data.variant.empathy_by_clue(*clue_type, clue_value);
            let empathy_bits = empathy.as_bits();

            let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = hand_cards
                .iter()
                .copied()
                .filter(|&idx| {
                    player_pov
                        .card_identity(idx)
                        .is_some_and(|id| (1u64 << id) & empathy_bits != 0)
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
                        turn: player_pov.table_state().current_turn,
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
pub fn still_needed_cards_mask(
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

/// Good-touch baseline mask the receiver should apply to a touched card.
///
/// Under H-Group's good-touch principle, a touched card is assumed to be eventually useful:
/// trash and identities already clued in another hand are filtered out. The returned mask
/// is the intersection of the clue's empathy with the still-needed set, minus identities
/// already clued elsewhere.
///
/// Returns `None` when the intersection is empty — in that case GTP would say the touched
/// card has no plausible useful identity (a bad-touch on every alternative), so no narrowing
/// is applied (the receiver cannot honestly conclude anything from GTP alone).
pub fn good_touch_baseline_mask(
    clue: &Clue,
    receiver: PlayerIndex,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> Option<VariantCardsBitField> {
    let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
    let still_needed = still_needed_cards_mask(table_state, static_data);
    let already_clued = already_clued_ids_mask(receiver, table_state, static_data);
    let mask = clue_mask & still_needed & !already_clued;
    if mask == 0 { None } else { Some(mask) }
}

/// Counts cards in `touched` that violate the good-touch principle.
///
/// A card is a bad touch if:
/// - its identity (truth, when visible to the searcher; public empathy otherwise) has no
///   overlap with still-needed cards, OR
/// - its exact identity is already clued in another player's hand, OR
/// - its exact identity is already clued in the receiver's hand on a card outside the
///   current `touched` set, OR
/// - another card earlier in `touched` shares the same exact identity (intra-clue
///   duplicate).
///
/// `truth` is the searcher's POV (root POV during tree search). For cards visible to the
/// searcher — including the receiver's hand when the clue-giver is the searcher — this
/// resolves to a singleton truth identity. Cards the searcher cannot see (fresh draws
/// during rollout, or the searcher's own cards under raw clue empathy) fall back to the
/// deck's public empathy.
///
/// Using truth here is essential: the clue itself narrows a touched card's *public*
/// empathy to the GTP-good set, which can hide a trash touch (e.g. clueing rank 1 to a
/// hand containing trash P1 makes its public empathy `{R1, G1, B1, P1}`, all overlapping
/// still-needed, even though the searcher knows it is P1). Truth-based detection sees
/// through this.
pub(crate) fn count_bad_touches(
    touched: &[CardDeckIndex],
    receiver: PlayerIndex,
    truth: &dyn PlayerPOV,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> usize {
    let still_needed = still_needed_cards_mask(table_state, static_data);

    // Build the already-clued mask using truth identities when the searcher can see
    // the card. `already_clued_ids_mask` uses only public singleton empathy, which
    // misses cards touched by a multi-value clue (e.g. a rank-1 clue leaves Y1 with
    // empathy {R1,Y1,G1,B1,P1}, so `known_card_id` returns None even though the
    // searcher knows it is Y1). Using truth.card_identity first mirrors the same
    // pattern used when scoring the `touched` cards below.
    let num_players = static_data.number_of_players as usize;
    let mut already_clued_other_hands: VariantCardsBitField = 0;
    for p in 0..num_players {
        if p == receiver {
            continue;
        }
        for &idx in table_state.hands[p].cards() {
            if table_state.clue_touched_cards & (1u64 << idx) != 0 {
                let bits = match truth.card_identity(idx) {
                    Some(id) => 1u64 << id,
                    None => match table_state.deck.get_global_empathy(idx).known_card_id() {
                        Some(id) => 1u64 << id,
                        None => continue,
                    },
                };
                already_clued_other_hands |= bits;
            }
        }
    }

    // Identities already known-clued in the receiver's own hand, excluding cards
    // currently being touched by this clue. A second touch onto an identity the
    // receiver already holds (as another clued card) is a bad touch too.
    let touched_mask: u64 = touched.iter().fold(0u64, |m, &i| m | (1u64 << i));
    let mut already_clued_receiver: VariantCardsBitField = 0;
    for &idx in table_state.hands[receiver].cards() {
        if (1u64 << idx) & touched_mask != 0 {
            continue;
        }
        if table_state.clue_touched_cards & (1u64 << idx) != 0 {
            let bits = match truth.card_identity(idx) {
                Some(id) => 1u64 << id,
                None => match table_state.deck.get_global_empathy(idx).known_card_id() {
                    Some(id) => 1u64 << id,
                    None => continue,
                },
            };
            already_clued_receiver |= bits;
        }
    }

    let already_clued = already_clued_other_hands | already_clued_receiver;

    // Track exact identities already accounted for among prior entries in `touched`,
    // so an intra-clue duplicate (e.g. two R1s in the same hand touched together)
    // counts as a bad touch on the second occurrence.
    let mut seen_in_touched: VariantCardsBitField = 0;
    let mut count = 0usize;
    for &idx in touched {
        let bits = match truth.card_identity(idx) {
            Some(id) => 1u64 << id,
            None => table_state.deck.get_global_empathy(idx).as_bits(),
        };
        let is_singleton = bits.count_ones() == 1;
        let dup_within_touched = is_singleton && (bits & seen_in_touched) != 0;
        let bad = bits & still_needed == 0 || bits & already_clued != 0 || dup_within_touched;
        if bad {
            count += 1;
        }
        if is_singleton {
            seen_in_touched |= bits;
        }
    }
    count
}

/// True when the clue may duplicate (in the receiver's hand) an identity the giver
/// likely already holds.
///
/// Two masks are built, each restricted to still-needed identities (trash filtered
/// out, since touching trash is already handled by `good_touch_penalty`):
///   - `touched_on_receiver`: truth identities of the cards touched by this clue on
///     the receiver's hand (giver sees these directly).
///   - `giver_held`: union of inferred identities of cards in the giver's own hand
///     that are either touched (clued previously) or carry a Play signal.
///
/// Fires (returns true) iff the two masks intersect.
pub(crate) fn is_potential_bad_touch(
    touched: &[CardDeckIndex],
    giver: PlayerIndex,
    truth: &dyn PlayerPOV,
    table_state: &TableState,
    static_data: &StaticGameData,
    team_knowledge: &TeamKnowledge,
) -> bool {
    let still_needed = still_needed_cards_mask(table_state, static_data);

    let mut touched_on_receiver: VariantCardsBitField = 0;
    for &idx in touched {
        if let Some(id) = truth.card_identity(idx) {
            touched_on_receiver |= 1u64 << id;
        }
    }
    touched_on_receiver &= still_needed;
    if touched_on_receiver == 0 {
        return false;
    }

    let giver_knowledge = team_knowledge.player(giver);
    let mut giver_held: VariantCardsBitField = 0;
    for &giver_card_idx in table_state.hands[giver].cards() {
        let is_touched = (table_state.clue_touched_cards >> giver_card_idx) & 1 != 0;
        let has_play_signal = giver_knowledge.has_play_signal(giver_card_idx);
        if !is_touched && !has_play_signal {
            continue;
        }
        let inferred = giver_knowledge
            .combined_possible_identities(giver_card_idx, table_state, &static_data.variant)
            .as_bits();
        giver_held |= inferred & still_needed;
    }

    touched_on_receiver & giver_held != 0
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
        player_pov,
        player_pov.table_state(),
        player_pov.static_data(),
    ) == 0
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
    use crate::engine::knowledge::player_knowledge::{
        PlayerKnowledge, knowledge_for_hand, knowledge_with_visible,
    };
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
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

    // ── clues_for_player_with_focus ────────────────────────────────────────

    #[test]
    fn clues_for_player_returns_empty_when_no_card_identity_is_known() {
        // Player 0 does not know card 10's identity → no clue can be computed.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

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
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

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
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R2, oldest
        table_state.update_with_draw_action(20); // R1, newest
        table_state.active_player_index = 0;

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

    // ── count_bad_touches ──────────────────────────────────────────────────

    /// Build a minimal-knowledge truth POV for tests: an empty PlayerKnowledge so
    /// `card_identity` falls back to the deck's public empathy. This matches the
    /// behaviour the old `count_bad_touches` had when it consulted the deck directly.
    fn make_truth_pov<'a>(
        knowledge: &'a PlayerKnowledge,
        team_knowledge: &'a TeamKnowledge,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> LightweightPlayerPOV<'a> {
        LightweightPlayerPOV::new(0, knowledge, team_knowledge, table_state, static_data)
    }

    #[test]
    fn count_bad_touches_zero_when_touched_card_still_needed() {
        // Card 10 has fresh (all-possible-IDs) deck empathy. Nothing has been played.
        // Every card ID overlaps with still-needed → no bad touch.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[10], 1, &truth, &table_state, &static_data),
            0
        );
    }

    #[test]
    fn count_bad_touches_one_when_touched_card_identity_already_played() {
        // Card 10 is in player 1's hand. The deck reveals it as R1. R1 is also played to the
        // stack → R1 is no longer needed → touching card 10 is a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Play R1 to the stack from a different deck position (card 0).
        table_state.update_with_draw_action(0);
        table_state.update_with_play_action_of_specific_card(
            0,
            NoVarCards::R1.as_variant_card_id(),
            &static_data,
        );

        // Draw card 10 for player 1 and mark it as R1 in the deck.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[10], 1, &truth, &table_state, &static_data),
            1
        );
    }

    #[test]
    fn count_bad_touches_one_when_exact_identity_clued_in_another_hand() {
        // Card 20 is in player 2's hand, exactly known as R1 and already clue-touched.
        // Card 10 is in player 1's hand (receiver). Its deck empathy overlaps with R1.
        // Because R1 is already committed elsewhere, touching card 10 is a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Card 20 → player 2, revealed as R1, marked as clue-touched.
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);
        table_state
            .deck
            .reveal_card(20, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state.clue_touched_cards |= 1 << 20;

        // Card 10 → player 1 (receiver) with fresh deck empathy (all IDs possible).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[10], 1, &truth, &table_state, &static_data),
            1
        );
    }

    #[test]
    fn count_bad_touches_detects_trash_via_truth_pov_after_clue_narrowing() {
        // Regression for the rank-1 → bad-touch P1 case from
        // tests/replays/should_clue_g1_by_color_avoiding_bad_touch.json.
        //
        // Card 10 is in player 1's hand. Its truth is P1 (purple-1). Purple is already
        // complete on the stacks, so P1 is trash. The receiver's *public* empathy on
        // card 10 may not even be a singleton (e.g. after a rank-1 clue narrows via GTP
        // to {R1, G1, B1, P1}, all overlapping still_needed). The truth POV — the
        // searcher who sees card 10 — knows it is P1 and must classify it as a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Fill the purple stack: play P1..P5 from spare deck positions.
        for (deck_idx, card) in [
            (0u8, NoVarCards::P1),
            (1, NoVarCards::P2),
            (2, NoVarCards::P3),
            (3, NoVarCards::P4),
            (4, NoVarCards::P5),
        ] {
            table_state.update_with_draw_action(deck_idx);
            table_state.update_with_play_action_of_specific_card(
                deck_idx,
                card.as_variant_card_id(),
                &static_data,
            );
        }

        // Draw card 10 into player 1's hand. The public deck empathy for card 10 still
        // includes many identities (no clue has narrowed it). The searcher, however,
        // knows it is a second P1.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        // Build a truth POV for player 0 (searcher) where card 10 is visible as a
        // second P1. Reveal does not mutate the stack, but does narrow the deck empathy
        // — we *don't* call reveal_card here because we want to simulate the case where
        // public empathy is *not* a singleton; only the searcher knows the identity via
        // their own `inferred_identities`.
        let mut knowledge = knowledge_for_hand(&[]);
        knowledge.inferred_identities[10] = Some(crate::game::card::CardIdentityMask::from_bits(
            1 << NoVarCards::P1.as_variant_card_id(),
        ));
        knowledge.visible_cards |= 1 << 10;
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(
            super::count_bad_touches(&[10], 1, &truth, &table_state, &static_data),
            1,
            "trash P1 in receiver's hand should be flagged via truth POV"
        );
    }

    #[test]
    fn count_bad_touches_one_when_two_touched_cards_share_identity_in_same_hand() {
        // Cards 10 and 11 are both in player 1's hand, both revealed as R1. A single
        // red clue touches both. R1 is still needed and not clued elsewhere, but the
        // two cards duplicate each other within `touched` — the second one is bad-touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state.update_with_draw_action(11);
        table_state
            .deck
            .reveal_card(11, NoVarCards::R1.as_variant_card_id(), &static_data.variant);

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[10, 11], 1, &truth, &table_state, &static_data),
            1,
            "intra-clue duplicate identities in same hand should count as one bad touch"
        );
    }

    #[test]
    fn count_bad_touches_one_when_identity_already_clued_in_receiver_hand() {
        // Card 10 is in player 1's hand, already clue-touched and exactly known as R1.
        // Card 11 (also player 1) is newly touched with truth R1 — duplicate against
        // the receiver's own already-clued R1.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state.clue_touched_cards |= 1 << 10;

        table_state.update_with_draw_action(11);
        table_state
            .deck
            .reveal_card(11, NoVarCards::R1.as_variant_card_id(), &static_data.variant);

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[11], 1, &truth, &table_state, &static_data),
            1,
            "newly touched card whose identity is already clued in receiver's own hand should be a bad touch"
        );
    }

    #[test]
    fn count_bad_touches_one_when_identity_clued_in_other_hand_via_multi_value_clue() {
        // Regression: `already_clued_ids_mask` only checked singleton public empathy, so a
        // card touched by a rank-1 clue (public empathy = all rank-1s, not a singleton) was
        // invisible to the bad-touch check even when the truth POV knew its exact identity.
        //
        // Setup: card 20 (player 2) is Y1 and was touched by a rank-1 clue, but the deck
        // empathy is NOT narrowed to a singleton (simulating a multi-value clue). The truth
        // POV (player 0) knows card 20 is Y1 via visible_cards / inferred_identities.
        // Card 10 (player 1, receiver) is also Y1 and known to the truth POV.
        // Touching card 10 must be flagged as a bad touch.
        use crate::game::card::CardIdentityMask;
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Card 20 → player 2: clue-touched but NOT deck-revealed (public empathy stays wide).
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);
        table_state.clue_touched_cards |= 1 << 20;

        // Card 10 → player 1 (receiver): also Y1.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        // Truth POV (player 0) sees both cards as Y1.
        let mut knowledge = knowledge_for_hand(&[]);
        let y1_id = NoVarCards::Y1.as_variant_card_id();
        let y1_mask = CardIdentityMask::from_bits(1 << y1_id);
        knowledge.inferred_identities[20] = Some(y1_mask);
        knowledge.visible_cards |= 1 << 20;
        knowledge.inferred_identities[10] = Some(y1_mask);
        knowledge.visible_cards |= 1 << 10;

        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(&[10], 1, &truth, &table_state, &static_data),
            1,
            "Y1 touched by a multi-value clue in another hand must be detected as already-clued via truth POV"
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

    // ── has_on_finesse_position ───────────────────────────────────────────

    #[test]
    fn returns_true_when_finesse_position_card_matches() {
        use crate::game::deck::unit_test_constants::novariant_constants::{
            NoVarCards, R1_MASK, R2_MASK,
        };

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Player 0 draws R1 (card 10) then R2 (card 11). Newest = R2.
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        // Reveal card identities in deck.
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state
            .deck
            .reveal_card(11, NoVarCards::R2.as_variant_card_id(), &static_data.variant);

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
        use crate::game::deck::unit_test_constants::novariant_constants::{
            NoVarCards, R1_MASK, R2_MASK,
        };

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state
            .deck
            .reveal_card(11, NoVarCards::R2.as_variant_card_id(), &static_data.variant);

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
        use crate::game::deck::unit_test_constants::novariant_constants::{NoVarCards, R1_MASK};

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10); // R1
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
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
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

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
        use crate::game::deck::unit_test_constants::novariant_constants::{
            NoVarCards, R1_MASK, R2_MASK, R3_MASK,
        };

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Draw order: R1 (oldest), R2, R3 (newest).
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(11); // R2
        table_state.update_with_draw_action(12); // R3
        table_state
            .deck
            .reveal_card(10, NoVarCards::R1.as_variant_card_id(), &static_data.variant);
        table_state
            .deck
            .reveal_card(11, NoVarCards::R2.as_variant_card_id(), &static_data.variant);
        table_state
            .deck
            .reveal_card(12, NoVarCards::R3.as_variant_card_id(), &static_data.variant);
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

    // ── is_potential_bad_touch ─────────────────────────────────────────────

    /// Common setup: 5-player no-variant game; giver = player 0 holds card 5 (clue-touched)
    /// in their hand, narrowed by inference to `giver_inferred_mask`. Receiver = player 1
    /// holds card 10 with truth = `receiver_truth`. Returns the pieces needed to invoke
    /// `is_potential_bad_touch(&[10], 0, ...)`.
    fn setup_pbt(
        giver_inferred_mask: u64,
        receiver_truth: crate::game::deck::unit_test_constants::novariant_constants::NoVarCards,
        giver_card_touched: bool,
        giver_play_signal: Option<VariantCardId>,
    ) -> (StaticGameData, TableState, TeamKnowledge, PlayerKnowledge) {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Giver (player 0) draws card 5.
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(5);
        if giver_card_touched {
            table_state.clue_touched_cards |= 1 << 5;
        }

        // Receiver (player 1) draws card 10, revealed as the chosen truth identity.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state
            .deck
            .reveal_card(10, receiver_truth.as_variant_card_id(), &static_data.variant);

        // Giver's knowledge: own_hand includes card 5; inferred identity set to the requested mask.
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        {
            let giver_k = team_knowledge.player_mut(0);
            giver_k.own_hand |= 1 << 5;
            giver_k.inferred_identities[5] =
                Some(crate::game::card::CardIdentityMask::from_bits(giver_inferred_mask));
            if let Some(id) = giver_play_signal {
                giver_k.add_signal(
                    5,
                    Signal::Play {
                        card_deck_index: 5,
                        committed_identity: id,
                    },
                );
            }
        }

        // Empty truth-side PlayerKnowledge — `card_identity(10)` falls back to deck empathy,
        // which was just narrowed to the singleton by `reveal_card`.
        let truth_knowledge = knowledge_for_hand(&[]);

        (static_data, table_state, team_knowledge, truth_knowledge)
    }

    #[test]
    fn pbt_fires_when_giver_holds_clued_singleton_matching_touched() {
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_mask = 1u64 << NoVarCards::R1.as_variant_card_id();
        let (static_data, table_state, team_knowledge, truth_knowledge) =
            setup_pbt(r1_mask, NoVarCards::R1, /*touched=*/ true, None);
        let truth = make_truth_pov(&truth_knowledge, &team_knowledge, &table_state, &static_data);

        assert!(super::is_potential_bad_touch(
            &[10],
            0,
            &truth,
            &table_state,
            &static_data,
            &team_knowledge,
        ));
    }

    #[test]
    fn pbt_fires_when_giver_holds_play_signaled_card_matching_touched() {
        // Play-signaled (but not clue-touched) giver-hand card still contributes to mask B.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_id = NoVarCards::R1.as_variant_card_id();
        let r1_mask = 1u64 << r1_id;
        let (static_data, table_state, team_knowledge, truth_knowledge) =
            setup_pbt(r1_mask, NoVarCards::R1, /*touched=*/ false, Some(r1_id));
        let truth = make_truth_pov(&truth_knowledge, &team_knowledge, &table_state, &static_data);

        assert!(super::is_potential_bad_touch(
            &[10],
            0,
            &truth,
            &table_state,
            &static_data,
            &team_knowledge,
        ));
    }

    #[test]
    fn pbt_does_not_fire_when_touched_id_is_trash() {
        // R1 is played to the red stack → trash → filtered out of both masks even though
        // the giver holds an R1-narrowed card and the clue touches an R1.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_id = NoVarCards::R1.as_variant_card_id();
        let r1_mask = 1u64 << r1_id;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Play R1 from a spare deck position to fill that slot of the red stack.
        table_state.update_with_draw_action(0);
        table_state.update_with_play_action_of_specific_card(0, r1_id, &static_data);

        // Giver (player 0) draws card 5, marked clue-touched, narrowed to R1.
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(5);
        table_state.clue_touched_cards |= 1 << 5;

        // Receiver (player 1) draws card 10, revealed as a second R1 (now trash).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state
            .deck
            .reveal_card(10, r1_id, &static_data.variant);

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        {
            let giver_k = team_knowledge.player_mut(0);
            giver_k.own_hand |= 1 << 5;
            giver_k.inferred_identities[5] =
                Some(crate::game::card::CardIdentityMask::from_bits(r1_mask));
        }
        let truth_knowledge = knowledge_for_hand(&[]);
        let truth = make_truth_pov(&truth_knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!super::is_potential_bad_touch(
            &[10],
            0,
            &truth,
            &table_state,
            &static_data,
            &team_knowledge,
        ));
    }

    #[test]
    fn pbt_does_not_fire_when_giver_hand_unclued_and_unsignaled() {
        // Giver's only hand card is unclued and has no play signal → mask B is empty.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_mask = 1u64 << NoVarCards::R1.as_variant_card_id();
        // Provide a narrowed inferred set, but leave touched=false / signal=None — the
        // gate should still skip this card and yield mask B = 0.
        let (static_data, table_state, team_knowledge, truth_knowledge) =
            setup_pbt(r1_mask, NoVarCards::R1, /*touched=*/ false, None);
        let truth = make_truth_pov(&truth_knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!super::is_potential_bad_touch(
            &[10],
            0,
            &truth,
            &table_state,
            &static_data,
            &team_knowledge,
        ));
    }

    #[test]
    fn pbt_does_not_fire_when_no_overlap_between_masks() {
        // Giver's clued card is narrowed to R1; clue touches Y1 on receiver.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_mask = 1u64 << NoVarCards::R1.as_variant_card_id();
        let (static_data, table_state, team_knowledge, truth_knowledge) =
            setup_pbt(r1_mask, NoVarCards::Y1, /*touched=*/ true, None);
        let truth = make_truth_pov(&truth_knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!super::is_potential_bad_touch(
            &[10],
            0,
            &truth,
            &table_state,
            &static_data,
            &team_knowledge,
        ));
    }
}
