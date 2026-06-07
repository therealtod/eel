use super::{get_clue_focus, get_finesse_position, prompt_connector_index};
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::MAX_UNIQUE_CARDS_IN_DECK;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::variant::Variant;
use smallvec::SmallVec;

/// <https://hanabi.github.io/beginner/minimum-clue-value-principle>/
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
    clue_receiver_player_index: PlayerIndex,
    touched_cards: &[CardDeckIndex],
    player_pov: &dyn PlayerPOV,
) -> bool {
    debug_assert!(!touched_cards.is_empty(), "No touched cards");

    // Condition 1: at least one touched card has an identity not yet in the gotten set.
    if touched_cards.iter().any(|&idx| {
        player_pov
            .card_identity(idx)
            .is_none_or(|id| !player_pov.is_gotten(id))
    }) {
        return true;
    }

    // Condition 2: more than one card transitions from "not going to play" to "going to play".
    count_newly_going_to_play(clue, clue_receiver_player_index, touched_cards, player_pov) > 1
}

/// Overlay describing how a candidate clue would change holder-level empathy and signals.
struct ClueOverlay<'a> {
    touched: &'a [CardDeckIndex],
    clue_mask: u64,
    /// Cards (and committed identities) that should be treated as receiving a `Signal::Play`
    /// as a result of the clue's positional inference (finesse). Each entry pins the card's
    /// empathy to a singleton and forces it into the "going to play" set unconditionally. For a
    /// detected finesse this includes both the connecting card on the teammate's finesse position
    /// and the focus itself (the 1-away card), since a finesse commits both.
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

/// Detects the play signals a candidate 1-away play clue introduces via positional inference,
/// so MCVP can count the cards it newly commits to play.
///
/// Two shapes commit a connecting card alongside the focus:
///
/// - **Finesse**: the connecting card sits on a teammate's finesse position (an unclued card,
///   where the teammate plays before the receiver). It is pinned to the connecting identity.
/// - **Prompt**: the connecting card is an already-clued, identity-ambiguous card in a teammate's
///   (or the receiver's own) hand — the holder is prompted to play their leftmost touched card
///   that could be the connector. [`prompt_connector_index`] locates and validates it.
///
/// When any connector is found the focus itself is also pinned to its (1-away) identity, mirroring
/// the two plays both a finesse and a prompt actually commit.
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
        if p == active {
            continue;
        }
        // Finesse: connector on a teammate's finesse position (teammate plays before receiver).
        if p != receiver
            && pov.static_data().plays_before(p, receiver, active)
            && let Some(finesse_idx) = get_finesse_position(p, pov)
            && pov.card_identity(finesse_idx) == Some(connecting_id)
        {
            result.push((finesse_idx, connecting_id));
            continue;
        }
        // Prompt: connector is an already-clued, ambiguous card in p's hand (excluding the focus).
        if let Some(connector_idx) = prompt_connector_index(connecting_id, p, focus, pov) {
            result.push((connector_idx, connecting_id));
        }
    }
    // A finesse or prompt commits TWO cards: the connector(s) pinned above AND the focus itself.
    // Only when a real connector was found (the precondition held) do we pin the focus to its
    // 1-away identity, so MCVP counts the focus as newly going-to-play alongside the connector.
    if !result.is_empty() {
        result.push((focus, focus_id));
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

        // Build this holder's entries first so own-hand card-count elimination can run across
        // them before they join the global set.
        let mut holder_entries: SmallVec<[(CardDeckIndex, u64, Option<VariantCardId>); 5]> =
            SmallVec::new();
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
                            && *card_deck_index == c
                        {
                            signaled = Some(*committed_identity);
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

            holder_entries.push((c, empathy, signaled));
        }

        prune_own_hand_exhausted(&mut holder_entries, pk, table_state, variant);
        entries.extend(holder_entries);
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
                if empathy.is_power_of_two() {
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

/// Own-hand card-count elimination across a single holder's cards.
///
/// `observable_*` reasoning only counts copies in *other* players' hands, so it never lets one of
/// a holder's own cards exhaust an identity for the holder's other cards. But when some of a
/// holder's cards are pinned to a singleton identity (by direct knowledge, a Play signal, or — in
/// the overlay — a candidate clue), those copies are located in this hand and can exhaust the
/// identity for the holder's remaining cards. Example: a clue rank-pins one card of an ambiguous
/// `{g3,g4}` pair to `g4`; with both `g4` copies then located, the sibling collapses to `g3`.
fn prune_own_hand_exhausted(
    entries: &mut SmallVec<[(CardDeckIndex, u64, Option<VariantCardId>); 5]>,
    pk: &PlayerKnowledge,
    table_state: &TableState,
    variant: &Variant,
) {
    // Copies of each identity already located in this holder's own hand (singleton empathy or a
    // committed Play signal).
    let mut own_located = [0u8; MAX_UNIQUE_CARDS_IN_DECK];
    for &(_, empathy, signaled) in entries.iter() {
        if let Some(id) = signaled {
            own_located[id] += 1;
        } else if empathy.is_power_of_two() {
            own_located[empathy.trailing_zeros() as usize] += 1;
        }
    }

    // Copies of each identity that could still occupy an unseen own-hand cell. A non-singleton
    // card cannot be identity X once X's own-hand copies are fully located by *other* cards.
    let remaining = pk.observable_remaining_copies(pk.visible_cards, table_state, variant);

    for (_, empathy, signaled) in entries.iter_mut() {
        if signaled.is_some() || empathy.count_ones() <= 1 {
            continue;
        }
        let mut bits = *empathy;
        while bits != 0 {
            let id = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            // This card is non-singleton, so it does not contribute to `own_located[id]`; no
            // self-exclusion needed. If every own-hand copy of `id` is located elsewhere in this
            // hand, `id` is impossible here.
            if own_located[id] >= remaining[id] {
                *empathy &= !(1u64 << id);
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::knowledge_with_visible;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::MAX_HAND_SIZE;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── is_minimal_clue_value_compliant ────────────────────────────────────

    #[test]
    fn mcvp_passes_when_at_least_one_touched_card_is_not_yet_gotten() {
        // Card 10 (R1) already gotten (clue-touched + known). Card 20 (R2) is new: its
        // identity R2 is not yet in gotten_cards → condition 1 passes.
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

        assert!(is_minimal_clue_value_compliant(&clue, 0, &touched, &pov));
    }

    #[test]
    fn mcvp_fails_when_all_touched_identities_already_gotten() {
        // Both touched cards carry identities (R1, R2) the team already has gotten, and each
        // holder's empathy is already singleton-narrowed — so both are going-to-play *before*
        // the clue. Re-touching them with rank-1 adds no new gotten identity (condition 1 fails)
        // and produces zero going-to-play transitions (condition 2 fails). MCVP rejects.
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

        assert!(!is_minimal_clue_value_compliant(&clue, 0, &touched, &pov));
    }

    #[test]
    fn mcvp_fails_when_touching_only_signal_touched_gotten_card() {
        // Card 10 (R1) carries a Signal::Play → it is signal-touched, and its identity is
        // known to the giver. Under the new definition R1 is already gotten even though it
        // was never clue-touched. Re-touching it with a clue fails condition 1.
        use crate::engine::convention::hgroup::signal::Signal;
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

        assert!(!is_minimal_clue_value_compliant(&clue, 0, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_for_single_untouched_card() {
        // Card 10 (R1) is not yet touched at all — its identity R1 is not gotten → passes.
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

        assert!(is_minimal_clue_value_compliant(&clue, 0, &touched, &pov));
    }

    #[test]
    fn mcvp_passes_via_condition1_when_clue_touches_fresh_card() {
        // Three R-suit cards in different hands: 10=R2 (touched, holder=P1),
        // 20=R3 (touched, holder=P2), 30=R1 (untouched, holder=P3). The clue touches card 30,
        // which carries an identity not yet in the gotten set — condition 1 alone passes MCVP.
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

        assert!(is_minimal_clue_value_compliant(&clue, 3, &touched, &pov));
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
            is_minimal_clue_value_compliant(&clue, 2, &touched, &pov),
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

    #[test]
    fn mcvp_passes_via_condition2_when_reclue_completes_a_prompt_chain() {
        // Stacks: R1, R2 played → R3 is directly playable, R4 is 1-away.
        // Cathy (P2) holds two already-color-clued, rank-ambiguous reds:
        //   slot 1 (newest) = R4 (card 21), slot 2 = R3 (card 20). Neither identity is known to
        //   Cathy (empathy still {red ranks}), so neither is going-to-play before the clue.
        // Both identities are gotten (P0 sees the touched cards' true ids) → condition 1 fails.
        // A *rank-4 re-clue* on card 21 (focus, away=1) prompts Cathy's leftmost remaining touched
        // red (card 20 = R3, playable) as the connector. The overlay pins R3 and the R4 focus, so
        // both newly become going-to-play → condition 2 passes.
        use crate::game::deck::unit_test_constants::novariant_constants::{
            NoVarCards, R3_MASK, R4_MASK,
        };

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(
            0,
            NoVarCards::R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            NoVarCards::R2.as_variant_card_id(),
            &static_data,
        );

        // Cathy (P2): draw R3 then R4 so newest-first is [21=R4, 20=R3].
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R3 → slot 1
        table_state.update_with_draw_action(21); // R4 → slot 1, R3 shifts to slot 2
        table_state.clue_touched_cards |= (1 << 20) | (1 << 21);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(20, R3_MASK), (21, R4_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Cathy holds both, with NO empathy narrowing (identities unknown to her).
        team_knowledge.player_mut(2).own_hand |= (1 << 20) | (1 << 21);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = Clue {
            clue_type: ClueType::Rank,
            clue_value: 4,
        };
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = smallvec::smallvec![21];

        assert!(
            is_minimal_clue_value_compliant(&clue, 2, &touched, &pov),
            "rank-4 prompt re-clue should pass MCVP via condition 2 (Cathy's R3 prompt + R4 focus \
             both newly become going-to-play)"
        );
        assert_eq!(
            super::count_newly_going_to_play(&clue, 2, &touched, &pov),
            2,
            "Cathy's R3 (prompted connector) and R4 (focus, chain-reachable through R3) both \
             transition from not-going-to-play to going-to-play"
        );
    }
}
