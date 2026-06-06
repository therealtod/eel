use super::get_clue_focus;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::MAX_HAND_SIZE;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use smallvec::SmallVec;

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
            if table_state.clue_touched_cards & (1u64 << idx) != 0
                && let Some(id) = table_state.deck.get_global_empathy(idx).known_card_id() {
                    mask |= 1u64 << id;
                }
        }
    }
    mask
}

/// Per-card good-touch baseline masks for every card touched by a clue, respecting the
/// no-duplication principle.
///
/// Under H-Group's good-touch principle a touched card is assumed eventually useful, so its
/// identity narrows to the clue's empathy intersected with the still-needed set, minus
/// identities already committed elsewhere (`good`). Applying that same `good` set to every
/// touched card is wrong when the set is smaller than the number of touched cards: each
/// still-needed identity completes the stack exactly once, so an identity is "good" for at
/// most one card. The focus has first claim on the communicated identity; once a touched card
/// is pinned to a single good identity, that identity is consumed and removed from the
/// remaining touched cards.
///
/// A touched card left with no remaining good identity is known trash: it can only be one of
/// the clue's already-played (or otherwise not-needed) identities. It is narrowed to
/// `clue_mask & !already_clued & !still_needed` so the holder treats it as a discard rather
/// than a phantom play of the duplicated identity.
///
/// Returns `(card_deck_index, mask)` pairs in focus-first order. Empty when GTP cannot narrow
/// anything (no still-useful identity for any touched card — `good` empty).
pub fn good_touch_receiver_masks(
    clue: &Clue,
    touched: &[CardDeckIndex],
    focus: Option<CardDeckIndex>,
    receiver: PlayerIndex,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> Vec<(CardDeckIndex, VariantCardsBitField)> {
    let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
    let still_needed = still_needed_cards_mask(table_state, static_data);
    // Identities already committed to a known-clued card: in another player's hand, or in
    // the receiver's own hand outside the set touched by this clue. A fresh touched card
    // cannot usefully be any of these (good touch forbids the duplicate), so they drop out
    // of the "still good" set. (`already_clued_ids_mask` covers only other hands; the
    // receiver's own already-known duplicates must be excluded too — otherwise a second
    // touch onto an identity the receiver already holds looks good and collapses two cards
    // onto it.)
    let touched_mask: VariantCardsBitField = touched.iter().fold(0, |m, &i| m | (1u64 << i));
    let mut already_clued = already_clued_ids_mask(receiver, table_state, static_data);
    for &idx in table_state.hands[receiver].cards() {
        if touched_mask & (1u64 << idx) != 0 {
            continue;
        }
        if table_state.clue_touched_cards & (1u64 << idx) != 0
            && let Some(id) = table_state.deck.get_global_empathy(idx).known_card_id() {
                already_clued |= 1u64 << id;
            }
    }
    let good = clue_mask & still_needed & !already_clued;
    if good == 0 {
        // No still-useful identity for any touched card: GTP cannot honestly narrow.
        return Vec::new();
    }
    // Identities a touched card can be if it is *not* a fresh useful card: the clue's
    // identities that are already clued elsewhere or no longer needed (trash/duplicates).
    let trash_mask = clue_mask & !already_clued & !still_needed;

    // A touched card already pinned to a single identity (e.g. a previously-clued card the
    // clue re-touches) claims that identity outright: consume it, and emit no narrowing
    // (intersecting it with its own identity is a no-op). Doing this first ensures the
    // remaining good identities the others can take are correct.
    let mut consumed: VariantCardsBitField = 0;
    let mut undetermined: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = SmallVec::new();
    for &idx in touched {
        match table_state.deck.get_global_empathy(idx).known_card_id() {
            Some(id) => consumed |= 1u64 << id,
            None => undetermined.push(idx),
        }
    }

    // Allocate the remaining good identities in priority order: the focus first, then the
    // other undetermined touched cards. Each still-needed identity is useful exactly once,
    // so a card pinned to a single good identity consumes it for the rest. A card left with
    // no remaining good identity is a known-trash duplicate.
    let ordered = focus
        .filter(|f| undetermined.contains(f))
        .into_iter()
        .chain(
            undetermined
                .iter()
                .copied()
                .filter(|&idx| Some(idx) != focus),
        );

    let mut result = Vec::with_capacity(undetermined.len());
    for idx in ordered {
        let available = good & !consumed;
        if available != 0 {
            result.push((idx, available));
            if available.is_power_of_two() {
                consumed |= available;
            }
        } else if trash_mask != 0 {
            result.push((idx, trash_mask));
        }
    }
    result
}

/// Counts cards in `touched` that *harmfully* violate the good-touch principle.
///
/// A card is a bad touch if:
/// - its identity (truth, when visible to the searcher; public empathy otherwise) has no
///   overlap with still-needed cards **and the receiver cannot tell it is trash**, OR
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
///
/// Truth-trash is *harmless*, however, when the receiver can resolve the card to **known
/// trash**. A clue may unavoidably touch a dead card (e.g. a rank-2 clue focusing the last
/// useful 2 also touches a now-dead 2): after focus-first good-touch allocation the receiver
/// pins that extra card to trash and simply discards it — it is never held as useful nor
/// misplayed, so it must not be penalised. This mirrors the receiver-side
/// [`good_touch_receiver_masks`] (see commit c8941de): a touched card whose receiver-resolved
/// mask shares no identity with `still_needed` is *not* a bad touch even though its truth is
/// dead. Cards the receiver cannot resolve via GTP (already pinned to a singleton, or a clue
/// with no still-useful identity at all) fall back to being counted, matching prior behaviour.
pub(crate) fn count_bad_touches(
    clue: &Clue,
    touched: &[CardDeckIndex],
    receiver: PlayerIndex,
    truth: &dyn PlayerPOV,
    table_state: &TableState,
    static_data: &StaticGameData,
) -> usize {
    let still_needed = still_needed_cards_mask(table_state, static_data);

    // Build the already-clued mask using truth identities when the searcher can see
    // the card. `already_clued_ids_mask` only checked singleton public empathy, so a
    // card touched by a multi-value clue (e.g. a rank-1 clue leaves Y1 with
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

    // Receiver-side good-touch resolution: the identity set the receiver assigns each touched
    // card after focus-first allocation. A truth-trash card whose resolved mask still overlaps
    // `still_needed` looks useful to the receiver (it will be held / possibly misplayed) — a
    // real bad touch. One whose mask is pure trash is known-trash to the receiver — harmless.
    // Cards absent from the result (pinned to a singleton, or a clue with no good identity at
    // all) are treated conservatively as "held useful", preserving prior behaviour.
    let focus = get_clue_focus(receiver, touched, truth);
    let receiver_masks =
        good_touch_receiver_masks(clue, touched, focus, receiver, table_state, static_data);
    let receiver_resolves_to_trash = |idx: CardDeckIndex| -> bool {
        receiver_masks
            .iter()
            .find(|(i, _)| *i == idx)
            .is_some_and(|(_, mask)| mask & still_needed == 0)
    };

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
        let is_singleton = bits.is_power_of_two();
        let dup_within_touched = is_singleton && (bits & seen_in_touched) != 0;
        let is_duplicate = bits & already_clued != 0 || dup_within_touched;
        // Dead-trash touches only harm when the receiver can't recognise them as trash.
        let harmful_trash = bits & still_needed == 0 && !receiver_resolves_to_trash(idx);
        if is_duplicate || harmful_trash {
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
/// out, since touching trash is already handled by `bad_touch_penalty`):
///   - `touched_on_receiver`: truth identities of the cards touched by this clue on
///     the receiver's hand (giver sees these directly).
///   - `giver_held`: union of inferred identities of cards in the giver's own hand
///     that are either touched (clued previously) or carry a Play signal.
///
/// Fires (returns true) if the two masks intersect.
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
        if !truth.is_touched(giver_card_idx) {
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
    clue: &Clue,
    clue_receiver_player_index: PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    count_bad_touches(
        clue,
        touched_cards,
        clue_receiver_player_index,
        player_pov,
        player_pov.table_state(),
        player_pov.static_data(),
    ) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_for_hand};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue_type::ClueType;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── count_bad_touches ──────────────────────────────────────────────────

    /// Build a minimal-knowledge truth POV for tests: an empty `PlayerKnowledge` so
    /// `card_identity` falls back to the deck's public empathy. This matches the
    /// behaviour the old `count_bad_touches` had when it consulted the deck directly.
    fn make_truth_pov<'a>(
        knowledge: &'a PlayerKnowledge,
        team_knowledge: &'a TeamKnowledge,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> LightweightPlayerPOV<'a> {
        // Test-only omniscient POV: every slot is treated as directly visible so
        // `card_identity` resolves via deck empathy rather than gating on a sight set.
        LightweightPlayerPOV::with_visible_cards(
            0,
            knowledge,
            team_knowledge,
            table_state,
            static_data,
            u64::MAX,
        )
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
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10],
                1,
                &truth,
                &table_state,
                &static_data
            ),
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
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10],
                1,
                &truth,
                &table_state,
                &static_data
            ),
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
        table_state.deck.reveal_card(
            20,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.clue_touched_cards |= 1 << 20;

        // Card 10 → player 1 (receiver) with fresh deck empathy (all IDs possible).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10],
                1,
                &truth,
                &table_state,
                &static_data
            ),
            1
        );
    }

    #[test]
    fn count_bad_touches_detects_trash_via_truth_pov_after_clue_narrowing() {
        // Regression for the rank-1 → bad-touch P1 case from
        // tests/fixtures/replays/should_clue_g1_by_color_avoiding_bad_touch.json.
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
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10],
                1,
                &truth,
                &table_state,
                &static_data
            ),
            1,
            "trash P1 in receiver's hand should be flagged via truth POV"
        );
    }

    #[test]
    fn count_bad_touches_zero_when_nonfocus_trash_resolves_to_known_trash() {
        // Regression for tests/fixtures/replays/should_clue_instead_of_discarding.json, mirroring
        // the receiver-side fix in commit c8941de on the evaluator side.
        //
        // Every rank-2 except B2 is played (R2, Y2, G2, P2 on the stacks), so among the
        // 2s only B2 is still useful. A rank-2 clue focuses the receiver's chop (truth B2,
        // still needed) and unavoidably also touches a now-dead G2. After focus-first
        // good-touch allocation the receiver pins B2 to the focus and the extra card to
        // pure trash {R2,Y2,G2,P2} — it is simply discarded, never held as useful nor
        // misplayed. So the dead G2 touch is *harmless* and must NOT count as a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Play every rank-2 except blue: R1,R2,Y1,Y2,G1,G2,P1,P2 (deck positions 0..8).
        for (deck_idx, card) in [
            (0u8, NoVarCards::R1),
            (1, NoVarCards::R2),
            (2, NoVarCards::Y1),
            (3, NoVarCards::Y2),
            (4, NoVarCards::G1),
            (5, NoVarCards::G2),
            (6, NoVarCards::P1),
            (7, NoVarCards::P2),
        ] {
            table_state.update_with_draw_action(deck_idx);
            table_state.update_with_play_action_of_specific_card(
                deck_idx,
                card.as_variant_card_id(),
                &static_data,
            );
        }

        // Receiver (player 1) draws card 10 (oldest → chop) then card 11 (newest). Neither
        // is publicly revealed, so the receiver's empathy is not a singleton and the
        // focus-first allocation treats both as undetermined. The chop (card 10) is touched,
        // so it is the clue focus.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(11);

        // The searcher (player 0) sees card 10 as B2 (still needed) and card 11 as a dead G2.
        let mut knowledge = knowledge_for_hand(&[]);
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(
            1 << NoVarCards::B2.as_variant_card_id(),
        ));
        knowledge.inferred_identities[11] = Some(CardIdentityMask::from_bits(
            1 << NoVarCards::G2.as_variant_card_id(),
        ));
        knowledge.visible_cards |= (1 << 10) | (1 << 11);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 2
                },
                &[10, 11],
                1,
                &truth,
                &table_state,
                &static_data
            ),
            0,
            "dead non-focus card that the receiver resolves to known trash must not be a bad touch"
        );
    }

    #[test]
    fn count_bad_touches_one_when_nonfocus_trash_resolves_to_useful_identity() {
        // Contrast to the harmless case above: when *two* rank-2s are still useful (B2 and
        // Y2), the focus-first allocation leaves the non-focus card with an ambiguous
        // still-useful mask {Y2,B2} — the receiver believes it is good and will hold (and
        // may eventually misplay) it. Its truth is a dead G2, so the touch IS harmful and
        // must count as a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Play R1,R2,G1,G2,P1,P2 and Y1 — leaving Y2 and B2 both still useful.
        for (deck_idx, card) in [
            (0u8, NoVarCards::R1),
            (1, NoVarCards::R2),
            (2, NoVarCards::G1),
            (3, NoVarCards::G2),
            (4, NoVarCards::P1),
            (5, NoVarCards::P2),
            (6, NoVarCards::Y1),
        ] {
            table_state.update_with_draw_action(deck_idx);
            table_state.update_with_play_action_of_specific_card(
                deck_idx,
                card.as_variant_card_id(),
                &static_data,
            );
        }

        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(11);

        let mut knowledge = knowledge_for_hand(&[]);
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(
            1 << NoVarCards::B2.as_variant_card_id(),
        ));
        knowledge.inferred_identities[11] = Some(CardIdentityMask::from_bits(
            1 << NoVarCards::G2.as_variant_card_id(),
        ));
        knowledge.visible_cards |= (1 << 10) | (1 << 11);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);

        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 2
                },
                &[10, 11],
                1,
                &truth,
                &table_state,
                &static_data
            ),
            1,
            "dead non-focus card the receiver resolves to a still-useful identity is a bad touch"
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
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.update_with_draw_action(11);
        table_state.deck.reveal_card(
            11,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10, 11],
                1,
                &truth,
                &table_state,
                &static_data
            ),
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
        table_state.deck.reveal_card(
            10,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );
        table_state.clue_touched_cards |= 1 << 10;

        table_state.update_with_draw_action(11);
        table_state.deck.reveal_card(
            11,
            NoVarCards::R1.as_variant_card_id(),
            &static_data.variant,
        );

        let knowledge = knowledge_for_hand(&[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let truth = make_truth_pov(&knowledge, &team_knowledge, &table_state, &static_data);
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[11],
                1,
                &truth,
                &table_state,
                &static_data
            ),
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
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10],
                1,
                &truth,
                &table_state,
                &static_data
            ),
            1,
            "Y1 touched by a multi-value clue in another hand must be detected as already-clued via truth POV"
        );
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
        table_state.deck.reveal_card(
            10,
            receiver_truth.as_variant_card_id(),
            &static_data.variant,
        );

        // Giver's knowledge: own_hand includes card 5; inferred identity set to the requested mask.
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        {
            let giver_k = team_knowledge.player_mut(0);
            giver_k.own_hand |= 1 << 5;
            giver_k.inferred_identities[5] = Some(crate::game::card::CardIdentityMask::from_bits(
                giver_inferred_mask,
            ));
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
        let truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

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
        let (static_data, table_state, team_knowledge, truth_knowledge) = setup_pbt(
            r1_mask,
            NoVarCards::R1,
            /*touched=*/ false,
            Some(r1_id),
        );
        let truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

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
        let truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

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
        let truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

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
        let truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

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
    fn pbt_does_not_fire_when_receiver_card_is_blinded() {
        // Unblinded: giver holds a clued R1; receiver's touched card is R1 → fires.
        // Blinded (receiver's deck index excluded from visible_cards): truth.card_identity
        // returns None → touched_on_receiver stays 0 → does NOT fire.
        // This is the actor-hand blinding invariant: a downstream player cannot use the
        // actor's hidden card identity to flag a bad touch.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let r1_mask = 1u64 << NoVarCards::R1.as_variant_card_id();
        let (static_data, table_state, team_knowledge, truth_knowledge) =
            setup_pbt(r1_mask, NoVarCards::R1, /*touched=*/ true, None);

        let omniscient_truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );
        assert!(
            super::is_potential_bad_touch(
                &[10],
                0,
                &omniscient_truth,
                &table_state,
                &static_data,
                &team_knowledge,
            ),
            "baseline: should fire with omniscient truth"
        );

        // Blinded truth: exclude deck index 10 (receiver's card) from visible_cards.
        let blinded_truth = LightweightPlayerPOV::with_visible_cards(
            0,
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
            !(1u64 << 10),
        );
        assert!(
            !super::is_potential_bad_touch(
                &[10],
                0,
                &blinded_truth,
                &table_state,
                &static_data,
                &team_knowledge,
            ),
            "should not fire when receiver's card is blinded"
        );
    }

    #[test]
    fn count_bad_touches_does_not_count_intra_clue_duplicate_when_identity_blinded() {
        // Two cards [10, 11] are touched, both R1 as known via direct sight.
        // Unblinded: card_identity returns R1 for both → second is detected as an
        // intra-clue duplicate (dup_within_touched) → count = 1.
        // Blinded: card_identity returns None for both; global deck empathy is not a
        // singleton (card not revealed in deck) → is_singleton = false → dup_within_touched
        // cannot fire → count = 0.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // Receiver (player 1) draws two cards at deck indices 10 and 11.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(11);

        // Truth POV's knowledge: direct-sight reveals both as R1, but the deck itself
        // has no reveal (get_global_empathy stays full-width), so blinding the cards
        // makes card_identity fall back to a multi-valued mask and return None.
        let mut truth_knowledge = knowledge_for_hand(&[]);
        truth_knowledge.update_with_revealed_card(10, NoVarCards::R1.as_variant_card_id());
        truth_knowledge.update_with_revealed_card(11, NoVarCards::R1.as_variant_card_id());

        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);

        let omniscient_truth = make_truth_pov(
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10, 11],
                1,
                &omniscient_truth,
                &table_state,
                &static_data
            ),
            1,
            "baseline: intra-clue duplicate detected with omniscient truth"
        );

        // Blinded truth: exclude both receiver cards from visible_cards.
        let blinded_truth = LightweightPlayerPOV::with_visible_cards(
            0,
            &truth_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
            !(1u64 << 10) & !(1u64 << 11),
        );
        assert_eq!(
            super::count_bad_touches(
                &Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1
                },
                &[10, 11],
                1,
                &blinded_truth,
                &table_state,
                &static_data
            ),
            0,
            "should not count intra-clue duplicate when card identities are blinded"
        );
    }
}
