use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
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
/// A clue is MCVP-compliant if either:
/// 1. It touches at least one card whose identity is not yet gotten (touched with known identity
///    anywhere on the team), OR
/// 2. It causes more than one card to transition from "not going to play" to "going to play",
///    where "going to play" is defined per-card from the holder's POV: their empathy on the card
///    is a non-empty subset of identities chain-reachable through other going-to-play cards'
///    committed identities (plus stack-immediately-playable identities). This subsumes finesses,
///    prompts, and delayed plays: a card becomes "going to play" when its holder, after the clue
///    (and any positional inference it triggers), can pin its candidate identities to a chain
///    that resolves through other committed cards.
pub fn is_minimal_clue_value_compliant(
    clue: &Clue,
    clue_receiver_player_index: &PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    debug_assert!(!touched_cards.is_empty(), "No touched cards");

    // Condition 1: at least one touched card has an identity not yet in the gotten set.
    if touched_cards.iter().any(|&idx| {
        player_pov
            .card_identity(idx)
            .map_or(true, |id| !player_pov.is_gotten(id))
    }) {
        return true;
    }

    // Condition 2: more than one card transitions from "not going to play" to "going to play".
    count_newly_going_to_play(clue, *clue_receiver_player_index, touched_cards, player_pov) > 1
}

/// Overlay describing how a candidate clue would change holder-level empathy and signals.
struct ClueOverlay<'a> {
    touched: &'a [CardDeckIndex],
    clue_mask: u64,
    /// Cards (and committed identities) that should be treated as receiving a `Signal::Play`
    /// as a result of the clue's positional inference (finesse). Each entry pins the card's
    /// empathy to a singleton and forces it into the "going to play" set unconditionally.
    finesse_signals: SmallVec<[(CardDeckIndex, VariantCardId); 4]>,
}

/// Counts how many cards transition from "not going to play" to "going to play" as a result of
/// the candidate clue.
pub(crate) fn count_newly_going_to_play(
    clue: &Clue,
    receiver: PlayerIndex,
    touched: &[CardDeckIndex],
    pov: &dyn PlayerPOV,
) -> usize {
    let clue_mask = pov.static_data().variant.empathy_for_clue(clue).as_bits();
    let finesse_signals = compute_finesse_signal_overlay(receiver, touched, pov);
    let overlay = ClueOverlay {
        touched,
        clue_mask,
        finesse_signals,
    };

    let pre = compute_going_to_play(pov, None);
    let post = compute_going_to_play(pov, Some(&overlay));

    (post & !pre).count_ones() as usize
}

/// Detects finesse signals introduced by the candidate clue. A 1-away focus with the connecting
/// card sitting on a teammate's finesse position (where the teammate plays before the receiver)
/// pins that finesse-position card to the connecting identity.
fn compute_finesse_signal_overlay(
    receiver: PlayerIndex,
    touched: &[CardDeckIndex],
    pov: &dyn PlayerPOV,
) -> SmallVec<[(CardDeckIndex, VariantCardId); 4]> {
    let mut result: SmallVec<[(CardDeckIndex, VariantCardId); 4]> = SmallVec::new();
    let Some(focus) = get_clue_focus(receiver, touched, pov) else {
        return result;
    };
    let Some(focus_id) = pov.card_identity(focus) else {
        return result;
    };
    if pov.away_value(focus_id) != Some(1) {
        return result;
    }
    if focus_id == 0 {
        return result;
    }
    let connecting_id = focus_id - 1;
    let active = pov.active_player_index();
    let num_players = pov.static_data().number_of_players as usize;
    for p in 0..num_players {
        if p == active || p == receiver {
            continue;
        }
        if !pov.static_data().plays_before(p, receiver, active) {
            continue;
        }
        if let Some(finesse_idx) = get_finesse_position(p, pov) {
            if pov.card_identity(finesse_idx) == Some(connecting_id) {
                result.push((finesse_idx, connecting_id));
            }
        }
    }
    result
}

/// Returns a bitmask over `CardDeckIndex` of cards that the team collectively expects to play.
///
/// A card `C` with holder `H` is "going to play" when **either**:
///   (a) `H` carries a `Signal::Play` on `C` (either via baseline signals or a tier-0 hypothesis
///       `AddSignal::Play`), or via the overlay's finesse signals — in which case `C`'s committed
///       identity contributes to the chain, **or**
///   (b) `H`'s effective empathy on `C` is non-empty and every identity in that empathy is
///       chain-reachable, where chain-reachability for an identity `X` means `X` is
///       immediately playable on the stacks, or every prerequisite back to a playable stack top
///       is committed by another going-to-play card with a singleton empathy.
///
/// The "going to play" set is computed by fixed-point iteration: cards with singleton empathy
/// contribute their identity to the committed-id pool, which can unlock further chain-reachable
/// cards on subsequent passes.
fn compute_going_to_play(pov: &dyn PlayerPOV, overlay: Option<&ClueOverlay>) -> u64 {
    let static_data = pov.static_data();
    let table_state = pov.table_state();
    let variant = &static_data.variant;
    let num_players = static_data.number_of_players as usize;
    let total_ids = variant.number_of_suits as usize * variant.stacks_size as usize;
    let playable = table_state.playable_cards(static_data);

    // Per-card snapshot under the optional overlay: (deck_idx, empathy, signaled_committed_id).
    let mut entries: SmallVec<[(CardDeckIndex, u64, Option<VariantCardId>); 25]> = SmallVec::new();

    for player_idx in 0..num_players {
        let pk = pov.team_knowledge().player(player_idx);
        for &c in table_state.hands[player_idx].cards() {
            let mut empathy = pk
                .combined_possible_identities(c, table_state, variant)
                .as_bits();

            // Baseline + tier-0-hypothesis Play signal commits the card.
            let mut signaled: Option<VariantCardId> = None;
            for sig in &pk.signals[c as usize] {
                if let Signal::Play {
                    committed_identity, ..
                } = sig
                {
                    signaled = Some(*committed_identity);
                    break;
                }
            }
            if signaled.is_none() {
                for h in &pk.hypotheses {
                    if h.tier != 0 {
                        continue;
                    }
                    for u in &h.immediate {
                        if let KnowledgeUpdate::AddSignal {
                            card_deck_index,
                            signal:
                                Signal::Play {
                                    committed_identity, ..
                                },
                        } = u
                        {
                            if *card_deck_index == c {
                                signaled = Some(*committed_identity);
                            }
                        }
                    }
                    if signaled.is_some() {
                        break;
                    }
                }
            }

            // Apply overlay: narrow touched cards by the clue mask, and apply finesse signals.
            if let Some(overlay) = overlay {
                if overlay.touched.contains(&c) {
                    empathy &= overlay.clue_mask;
                }
                for &(sig_idx, committed_id) in &overlay.finesse_signals {
                    if sig_idx == c {
                        signaled = Some(committed_id);
                    }
                }
            }

            entries.push((c, empathy, signaled));
        }
    }

    let mut playing_cards: u64 = 0;
    let mut committed_ids: u64 = 0;

    loop {
        let prev_playing = playing_cards;
        let prev_committed = committed_ids;

        for &(c, empathy, signaled) in &entries {
            if (playing_cards >> c) & 1 != 0 {
                continue;
            }

            let is_playing = if let Some(committed_id) = signaled {
                committed_ids |= 1u64 << committed_id;
                true
            } else if empathy != 0
                && (0..total_ids).all(|x| {
                    (empathy >> x) & 1 == 0
                        || is_chain_reachable_via_playing(x, playable, committed_ids, variant)
                })
            {
                if empathy.count_ones() == 1 {
                    committed_ids |= empathy;
                }
                true
            } else {
                false
            };

            if is_playing {
                playing_cards |= 1u64 << c;
            }
        }

        if playing_cards == prev_playing && committed_ids == prev_committed {
            break;
        }
    }

    playing_cards
}

/// True when identity `id` is reachable on the stacks given `playable` (immediate playables) and
/// `committed` (identities pinned by going-to-play cards with singleton empathy).
fn is_chain_reachable_via_playing(
    id: usize,
    playable: u64,
    committed: u64,
    variant: &Variant,
) -> bool {
    let mut current = id;
    loop {
        if (playable >> current) & 1 != 0 {
            return true;
        }
        match variant.prerequisite(current) {
            None => return false,
            Some(prereq) => {
                if (committed >> prereq) & 1 == 0 {
                    return false;
                }
                current = prereq;
            }
        }
    }
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
        if table_state.clue_touched_cards & (1u64 << idx) != 0 {
            if let Some(id) = table_state.deck.get_global_empathy(idx).known_card_id() {
                already_clued |= 1u64 << id;
            }
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
            if available.count_ones() == 1 {
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
        let is_singleton = bits.count_ones() == 1;
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
/// out, since touching trash is already handled by `good_touch_penalty`):
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
    clue_receiver_player_index: &PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    count_bad_touches(
        clue,
        touched_cards,
        *clue_receiver_player_index,
        player_pov,
        player_pov.table_state(),
        player_pov.static_data(),
    ) == 0
}

/// True when `card_id` could be the focus of *some* H-Group play clue tech in the current
/// position — i.e. the team could clue it to be played, now or via a connecting chain. This
/// unifies the four play clue techs by the precondition they all share:
///
/// - **DirectPlayClue** — the identity is immediately playable on the stacks (`away_value == 0`).
/// - **DelayedPlayClue / SimplePrompt** — the focus is `n`-away and the connecting cards down to
///   the stack are present (clued / known) in some hand.
/// - **SimpleFinesse** — the focus is 1-away and the connecting card sits unclued on a teammate's
///   finesse position.
///
/// Every one of them requires each connecting card between the stack top and `card_id` to be
/// *physically present in some hand*, so the chain can actually be set up and played. We
/// approximate the techs by that shared precondition, checked against `pov` (the searcher's
/// truth POV, which sees every hand). A card already on the stacks (`away_value == None`) is not
/// focusable.
pub fn is_play_clue_focusable(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
    let Some(away) = pov.away_value(card_id) else {
        return false; // already played
    };
    if away == 0 {
        return true; // immediately playable (DirectPlayClue)
    }
    // Delayed / prompt / finesse: every connecting card down to the playable frontier must exist
    // in some hand for the team to set the chain up.
    let table_state = pov.table_state();
    let num_players = pov.static_data().number_of_players as usize;
    (1..=away as usize).all(|offset| {
        let connecting_id = card_id - offset;
        (0..num_players).any(|p| {
            table_state.hands[p]
                .cards()
                .iter()
                .any(|&idx| pov.card_identity(idx) == Some(connecting_id))
        })
    })
}

/// https://hanabi.github.io/docs/beginner/#the-save-principle  (Save Principle)
///
/// The Save Principle says the team must protect from being discarded every card it cannot
/// afford to lose. Its protected set has three parts:
///
/// 1. **Playable cards** — any identity a play clue tech could focus ([`is_play_clue_focusable`]:
///    immediately playable, or focusable via a delayed play / prompt / finesse). Discarding a
///    card the team could just play away is a direct tempo and score loss.
/// 2. **Critical cards** (universal): the last live copy of a still-needed identity. Losing it
///    caps the achievable score, so it must be saved regardless of variant.
/// 3. **A variant-dependent set** ([`Variant::always_save_cards`]): identities saved *even before
///    they become critical*. In the base game these are the 2s — only two copies exist and both
///    are commonly exposed early, so the team saves them pre-emptively. (5s are single-copy and
///    therefore always critical, so they fall under part 2.)
///
/// An identity already on the stacks needs no saving, so `away_value == None` excludes it.
///
/// **Precedence over Good Touch:** the Save Principle outranks the Good Touch Principle. When a
/// save of a protected card on chop cannot avoid bad-touching another card, the team still
/// gives it — the bad touch is *forced* by the higher obligation, not chosen. The evaluator
/// reflects this by discounting bad touches incurred by a forced save (see
/// [`is_forced_save`] and `DefaultEvaluator::forced_save_bad_touch_penalty`).
pub fn is_save_principle_protected(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
    // Already on the stacks -> nothing to protect.
    let Some(away) = pov.away_value(card_id) else {
        return false;
    };
    // Playable now, or focusable by a play clue tech (delayed / prompt / finesse).
    if is_play_clue_focusable(card_id, pov) {
        return true;
    }
    // Critical: last live copy of a still-needed card.
    if pov.is_critical_card_id(card_id) {
        return true;
    }
    // Variant always-save set (the 2s in the base game), while still relevant to the stacks.
    away > 0 && (1u64 << card_id) & pov.static_data().variant.always_save_cards != 0
}

/// Save Principle, clue-level view: returns true when `clue` (via its `touched` set) saves a
/// Save-Principle-protected card sitting on `receiver`'s chop — i.e. the receiver's chop card is
/// protected ([`is_save_principle_protected`]) and the clue touches it.
///
/// `pov` must be able to resolve the receiver's chop identity (the searcher's root/truth POV,
/// which sees other players' hands). A card the POV cannot identify is treated as not-a-save.
///
/// This is the predicate the evaluator uses to recognise a *forced* bad touch: a clue that
/// honours the Save Principle and unavoidably bad-touches collateral is less culpable than a
/// deliberate bad touch, because the Save Principle outranks Good Touch.
pub fn is_forced_save(
    touched: &[CardDeckIndex],
    receiver: PlayerIndex,
    pov: &dyn PlayerPOV,
) -> bool {
    let Some(chop) = get_chop_index(receiver, pov) else {
        return false;
    };
    if !touched.contains(&chop) {
        return false;
    }
    pov.card_identity(chop)
        .is_some_and(|chop_id| is_save_principle_protected(chop_id, pov))
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

/// Returns `true` while the game is still in the **Early Game** phase.
///
/// The Early Game lasts until the first time any player performs a
/// [`DiscardChop`] — i.e. discards their chop, the oldest unclued (and
/// unsignalled) card in hand. Giving clues, playing cards, and even discarding
/// *known trash* all keep the game in the Early Game; only a genuine chop
/// discard ends it.
///
/// Detection replays the recorded turn `history`: each consecutive pair of
/// snapshots brackets exactly one action, and a discard is recognised when the
/// discard pile grew by one with no new strike (a misplay also fills the pile
/// but adds a strike). The card that left the actor's hand is then tested
/// against [`DiscardChop`] from the actor's POV *at that turn*, so the
/// definition stays single-sourced with the tech itself.
///
/// `current` is the live table state, paired against the final recorded
/// snapshot so the most recent action — which has no trailing snapshot yet — is
/// also considered.
#[must_use]
pub fn is_early_game(
    history: &[GameStateSnapshot],
    current: &TableState,
    static_data: &StaticGameData,
) -> bool {
    let recorded = history.windows(2).map(|w| (&w[0], &w[1].table_state));
    let latest = history.last().map(|before| (before, current));
    !recorded
        .chain(latest)
        .any(|(before, after)| transition_is_chop_discard(before, after, static_data))
}

/// Returns `true` if the single action bracketed by `before` (state before the
/// action) and `after` (state after it) was a chop discard by `before`'s active
/// player. See [`is_early_game`].
fn transition_is_chop_discard(
    before: &GameStateSnapshot,
    after: &TableState,
    static_data: &StaticGameData,
) -> bool {
    let before_state = &before.table_state;
    let discarded_one = after.discard_pile.size() == before_state.discard_pile.size() + 1;
    let no_new_strike = after.strike_tokens == before_state.strike_tokens;
    if !(discarded_one && no_new_strike) {
        return false;
    }

    let actor = before_state.active_player_index();
    let after_cards = after.hands[actor].cards();
    let Some(discarded) = before_state.hands[actor]
        .cards()
        .iter()
        .copied()
        .find(|idx| !after_cards.contains(idx))
    else {
        return false;
    };

    let pov = before.player_pov(actor, static_data);
    DiscardChop.matches_action(
        &GameAction::Discard {
            player_index: actor,
            card_deck_index: discarded,
            turn: before_state.current_turn,
        },
        &[],
        &pov,
    )
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
        // tests/fixtures/regression/should_clue_g1_by_color_avoiding_bad_touch/replay.json.
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
        // Regression for tests/replays/should_clue_instead_of_discarding.json, mirroring
        // the receiver-side fix in commit c8941de on the evaluator side.
        //
        // Every rank-2 except B2 is played (R2, Y2, G2, P2 on the stacks), so among the
        // 2s only B2 is still useful. A rank-2 clue focuses the receiver's chop (truth B2,
        // still needed) and unavoidably also touches a now-dead G2. After focus-first
        // good-touch allocation the receiver pins B2 to the focus and the extra card to
        // pure trash {R2,Y2,G2,P2} — it is simply discarded, never held as useful nor
        // misplayed. So the dead G2 touch is *harmless* and must NOT count as a bad touch.
        use crate::game::card::CardIdentityMask;
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
        use crate::game::card::CardIdentityMask;
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

    // ── is_minimal_clue_value_compliant ────────────────────────────────────

    #[test]
    fn mcvp_passes_when_at_least_one_touched_card_is_not_yet_gotten() {
        // Card 10 (R1) already gotten (clue-touched + known). Card 20 (R2) is new: its
        // identity R2 is not yet in gotten_cards → condition 1 passes.
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(20); // R2
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= (1 << 10) | (1 << 20);
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
    fn mcvp_fails_when_all_touched_identities_already_gotten() {
        // Both touched cards carry identities (R1, R2) the team already has gotten, and each
        // holder's empathy is already singleton-narrowed — so both are going-to-play *before*
        // the clue. Re-touching them with rank-1 adds no new gotten identity (condition 1 fails)
        // and produces zero going-to-play transitions (condition 2 fails). MCVP rejects.
        use crate::game::card::CardIdentityMask;
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.update_with_draw_action(20); // R2
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= (1 << 10) | (1 << 20);

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pk = team_knowledge.player_mut(1);
        pk.own_hand |= (1 << 10) | (1 << 20);
        pk.inferred_identities[10] = Some(CardIdentityMask::from_bits(R1_MASK));
        pk.inferred_identities[20] = Some(CardIdentityMask::from_bits(R2_MASK));
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
    fn mcvp_fails_when_touching_only_signal_touched_gotten_card() {
        // Card 10 (R1) carries a Signal::Play → it is signal-touched, and its identity is
        // known to the giver. Under the new definition R1 is already gotten even though it
        // was never clue-touched. Re-touching it with a clue fails condition 1.
        use crate::engine::convention::hgroup::signal::Signal;
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::{NoVarCards, R1_MASK};

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0;
        // Card 10 is NOT clue-touched, but carries a Signal::Play (signal-touched).

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pk = team_knowledge.player_mut(1);
        pk.own_hand |= 1 << 10;
        pk.add_signal(
            10,
            Signal::Play {
                card_deck_index: 10,
                committed_identity: NoVarCards::R1.as_variant_card_id(),
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0, // red
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![10];

        assert!(!is_minimal_clue_value_compliant(&clue, &0, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_for_single_untouched_card() {
        // Card 10 (R1) is not yet touched at all — its identity R1 is not gotten → passes.
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::R1_MASK;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= 1 << 10;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![10];

        assert!(is_minimal_clue_value_compliant(&clue, &0, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_via_condition1_when_clue_touches_fresh_card() {
        // Three R-suit cards in different hands: 10=R2 (touched, holder=P1),
        // 20=R3 (touched, holder=P2), 30=R1 (untouched, holder=P3). The clue touches card 30,
        // which carries an identity not yet in the gotten set — condition 1 alone passes MCVP.
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::{
            R1_MASK, R2_MASK, R3_MASK,
        };

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);
        table_state.active_player_index = 3;
        table_state.update_with_draw_action(30);
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= (1 << 10) | (1 << 20);

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, R3_MASK), (30, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= 1 << 10;
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(3).own_hand |= 1 << 30;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![30];

        assert!(is_minimal_clue_value_compliant(&clue, &3, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_via_condition2_when_clue_completes_a_finesse_chain() {
        // Stacks: R1 unplayed. Bob (P1) has R1 on his finesse position (slot 1, untouched).
        // Cathy (P2) has R2 already clue-touched, with holder empathy narrowed to {R2}.
        // R2's identity is in the gotten set (P0 sees the touched card's true identity) so
        // condition 1 fails. Re-touching R2 by a color-red clue triggers the SimpleFinesse
        // interpretation: Bob's slot 1 commits to R1. Both Cathy's R2 (focus, going-to-play
        // via the new R1 commitment) and Bob's R1 (signaled) transition into going-to-play,
        // satisfying condition 2.
        use crate::engine::convention::hgroup::signal::Signal as _Signal;
        use crate::game::card::CardIdentityMask;
        use crate::game::clue::Clue;
        use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let _ = _Signal::Play {
            card_deck_index: 0,
            committed_identity: 0,
        }; // suppress unused-import lint
        let mut table_state = initial_five_players_table_state();

        // Bob (P1) draws R1 to slot 1 (newest unclued = finesse position).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        // Cathy (P2) draws R2; we'll mark it clue-touched.
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.clue_touched_cards |= 1 << 20;
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Bob: R1 in own hand, no empathy narrowing.
        team_knowledge.player_mut(1).own_hand |= 1 << 10;
        // Cathy: R2 in own hand, empathy narrowed to {R2} (singleton) — already gotten.
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Pre-clue: R2 (Cathy, card 20) is in gotten_cards, so condition 1 will not save us.
        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![20];

        assert!(
            is_minimal_clue_value_compliant(&clue, &2, &touched, &pov),
            "finesse re-clue should pass MCVP via condition 2 (Bob's R1 + Cathy's R2 both \
             newly become going-to-play)"
        );
        assert_eq!(
            super::count_newly_going_to_play(&clue, 2, &touched, &pov),
            2,
            "Bob's R1 (signaled via finesse) and Cathy's R2 (chain-reachable through R1) both \
             transition from not-going-to-play to going-to-play"
        );
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
        use crate::game::deck::unit_test_constants::novariant_constants::{
            NoVarCards, R1_MASK, R2_MASK,
        };

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
        use crate::game::deck::unit_test_constants::novariant_constants::{NoVarCards, R1_MASK};

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
            !0u64 & !(1u64 << 10),
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
            !0u64 & !(1u64 << 10) & !(1u64 << 11),
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

    // ── Save Principle (is_save_principle_protected / is_forced_save) ────────

    use crate::game::deck::unit_test_constants::novariant_constants::{R3_MASK, R4_MASK};

    /// Build a one-card chop for player 1 (receiver) visible to player 0 (observer), optionally
    /// discarding `discard_id` copies first to manipulate criticality. Returns a ready POV plus
    /// the owning table_state/knowledge so the caller can keep them alive.
    fn receiver_chop_pov(
        chop_mask: u64,
        extra: &[(u8, u64)],
        discards: &[usize],
    ) -> (TableState, PlayerKnowledge, TeamKnowledge) {
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // oldest = chop
        for &(idx, _) in extra {
            table_state.update_with_draw_action(idx);
        }
        table_state.active_player_index = 0;
        for &id in discards {
            table_state.discard_pile.add_card_with_id(id);
        }
        let mut visible = vec![(10u8, chop_mask)];
        visible.extend_from_slice(extra);
        let knowledge = knowledge_with_visible(0, &visible);
        let team_knowledge =
            TeamKnowledge::new(NOVAR_5_PLAYERS_STATIC_GAME_DATA.number_of_players as usize);
        (table_state, knowledge, team_knowledge)
    }

    #[test]
    fn save_principle_protects_critical_card() {
        // R4 (id 3) has 2 copies; discard one → critical → protected.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R4_MASK, &[], &[3]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        // R4 = id 3.
        assert!(super::is_save_principle_protected(3, &pov));
    }

    #[test]
    fn save_principle_protects_uncritical_two() {
        // R2 (id 1) has 2 live copies — not critical — but is in the base variant's always-save
        // set, so the Save Principle still protects it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R2_MASK, &[], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(
            !pov.is_critical_card_id(1),
            "R2 should not be critical here"
        );
        assert!(super::is_save_principle_protected(1, &pov));
    }

    #[test]
    fn save_principle_ignores_uncritical_non_save_card() {
        // R3 (id 2) has 2 live copies, is not critical, is not in the always-save set, and has no
        // connecting cards present, so it is not play-clue-focusable either.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R3_MASK, &[], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(!super::is_save_principle_protected(2, &pov));
    }

    #[test]
    fn is_play_clue_focusable_true_for_directly_playable() {
        // R1 (id 0) on empty stacks is immediately playable (away == 0).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R1_MASK, &[], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(super::is_play_clue_focusable(0, &pov));
    }

    #[test]
    fn is_play_clue_focusable_true_for_delayed_with_connectors_present() {
        // R3 (id 2) is 2-away on empty stacks; its connectors R2 (id 1) and R1 (id 0) are present
        // in a hand, so the team could set up a delayed play / prompt / finesse onto it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R3_MASK, &[(20, R1_MASK), (21, R2_MASK)], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(super::is_play_clue_focusable(2, &pov));
        // ...and the Save Principle therefore protects it even though it is neither critical nor a 2.
        assert!(super::is_save_principle_protected(2, &pov));
    }

    #[test]
    fn is_play_clue_focusable_false_when_a_connector_is_missing() {
        // R3 is 2-away but only R1 is present (R2 absent), so the chain cannot be set up.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R3_MASK, &[(20, R1_MASK)], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(!super::is_play_clue_focusable(2, &pov));
    }

    #[test]
    fn save_principle_protects_directly_playable_card() {
        // A directly-playable R1 on chop is protected (a play clue tech could focus it).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R1_MASK, &[], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(super::is_save_principle_protected(0, &pov));
        assert!(super::is_forced_save(&[10], 1, &pov));
    }

    #[test]
    fn is_forced_save_true_when_protected_chop_is_touched() {
        // Receiver chop is critical R4; a clue touching the chop is a forced save.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R4_MASK, &[(20, R3_MASK)], &[3]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        // Touch both the chop (10) and the interior R3 (20).
        assert!(super::is_forced_save(&[10, 20], 1, &pov));
    }

    #[test]
    fn is_forced_save_false_when_chop_not_touched() {
        // Critical chop exists but the clue does not touch it (only the interior card).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R4_MASK, &[(20, R3_MASK)], &[3]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(!super::is_forced_save(&[20], 1, &pov));
    }

    #[test]
    fn is_forced_save_false_when_chop_not_protected() {
        // Chop is an uncritical non-save R3 — touching it is not a forced save.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (ts, k, tk) = receiver_chop_pov(R3_MASK, &[], &[]);
        let pov = LightweightPlayerPOV::new(0, &k, &tk, &ts, &static_data);
        assert!(!super::is_forced_save(&[10], 1, &pov));
    }

    // ── is_early_game ─────────────────────────────────────────────────────────

    /// Build a `TableState` where player 0 holds `cards` (oldest→newest), all
    /// unclued, with `clue_touched_cards` set from `touched`.
    fn state_with_hand(cards: &[u8], touched: &[u8]) -> TableState {
        let mut table_state = initial_five_players_table_state();
        for &idx in cards {
            table_state.update_with_draw_action(idx);
        }
        for &idx in touched {
            table_state.clue_touched_cards |= 1 << idx;
        }
        table_state
    }

    fn snapshot(table_state: &TableState) -> GameStateSnapshot {
        GameStateSnapshot::new(
            table_state.clone(),
            TeamKnowledge::new(NOVAR_5_PLAYERS_STATIC_GAME_DATA.number_of_players as usize),
        )
    }

    /// Apply a discard of `card` by the active player to a clone of `table_state`.
    fn after_discard(table_state: &TableState, card: u8) -> TableState {
        let mut after = table_state.clone();
        let actor = after.active_player_index();
        after.hands[actor].remove_card(card);
        after.discard_pile.add_card();
        after
    }

    #[test]
    fn early_game_holds_with_no_history() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let current = state_with_hand(&[10, 20, 30], &[]);
        assert!(is_early_game(&[], &current, &static_data));
    }

    #[test]
    fn chop_discard_in_history_ends_early_game() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        // Player 0 holds [10, 20, 30] all unclued → chop = 10.
        let before = state_with_hand(&[10, 20, 30], &[]);
        let after = after_discard(&before, 10); // discards the chop
        let current = state_with_hand(&[20, 30], &[]);
        let history = vec![snapshot(&before), snapshot(&after)];
        assert!(!is_early_game(&history, &current, &static_data));
    }

    #[test]
    fn off_chop_trash_discard_keeps_early_game() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        // Chop = 10 (oldest unclued); discarding the newest card (30) is not a
        // DiscardChop, so the Early Game continues.
        let before = state_with_hand(&[10, 20, 30], &[]);
        let after = after_discard(&before, 30);
        let history = vec![snapshot(&before), snapshot(&after)];
        assert!(is_early_game(&history, &after, &static_data));
    }

    #[test]
    fn misplay_keeps_early_game() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let before = state_with_hand(&[10, 20, 30], &[]);
        // A misplay also fills the discard pile but adds a strike — not a discard.
        let mut after = after_discard(&before, 10);
        after.strike_tokens += 1;
        let history = vec![snapshot(&before), snapshot(&after)];
        assert!(is_early_game(&history, &after, &static_data));
    }

    #[test]
    fn chop_discard_as_latest_action_ends_early_game() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        // Only the pre-action snapshot is recorded; the chop discard is detected
        // by pairing the last snapshot against the live `current` state.
        let before = state_with_hand(&[10, 20, 30], &[]);
        let current = after_discard(&before, 10);
        let history = vec![snapshot(&before)];
        assert!(!is_early_game(&history, &current, &static_data));
    }
}
