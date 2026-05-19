use smallvec::smallvec;

use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{
    AltGroupKey, Hypothesis, HypothesisId, HypothesisSet, KnowledgeUpdate, PendingTrigger,
};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue whose focus card is not immediately playable but will become playable once all
/// connecting cards, which are already globally known to the team are played.
pub struct DelayedPlayClue;

impl DelayedPlayClue {
    /// Core delayed play detection: checks if the focus card is not immediately playable but will
    /// become playable once connecting cards are played (and all connecting cards are globally known).
    fn is_delayed_play_situation(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
        if let Some(away_value) = pov.away_value(card_id) {
            away_value > 0
                && !pov.is_gotten(card_id)
                && Self::connecting_cards_are_known(card_id, away_value, pov)
        } else {
            false
        }
    }

    fn connecting_cards_are_known(
        card_id: VariantCardId,
        away_value: u8,
        pov: &dyn PlayerPOV,
    ) -> bool {
        let num_players = pov.static_data().number_of_players as usize;
        let table_state = pov.table_state();
        let variant = &pov.static_data().variant;
        let playable_mask = table_state.playable_cards(pov.static_data());
        let active = pov.active_player_index();

        (1..=away_value as usize).all(|offset| {
            let connecting_id = card_id - offset;
            let connecting_bit = 1u64 << connecting_id;
            (0..num_players).any(|p| {
                let pk = pov.team_knowledge().player(p);
                table_state.hands[p].cards().iter().any(|&idx| {
                    if !pov.is_touched(idx) {
                        return false;
                    }
                    // Strict path: holder knows the exact identity and it equals connecting_id.
                    if pov.is_identity_known_to_holder(idx)
                        && pov.card_identity(idx) == Some(connecting_id)
                    {
                        return true;
                    }
                    // Receiver-friendly path: holder's empathy on the card is a subset of
                    // currently-playable identities (i.e. the holder treats it as a known
                    // playable) and `connecting_id` is one of those possibilities. Covers
                    // the case "I have a known-playable 1 of unknown color" — a valid
                    // connecting card for a delayed play through any matching 2.
                    //
                    // Not applicable to the active player's own cards: they see their own
                    // possibilities (not truth), so `connecting_id` being one of several
                    // possibilities does not guarantee the card actually IS connecting_id.
                    if p == active {
                        return false;
                    }
                    // The giver can see other players' cards directly. If the giver can
                    // determine the actual identity and it doesn't match connecting_id,
                    // the card is provably not the connector — even if the holder's empathy
                    // includes connecting_id as a possibility.
                    if let Some(giver_id) = pov.card_identity(idx) {
                        if giver_id != connecting_id {
                            return false;
                        }
                    }
                    let possibilities = pk
                        .combined_possible_identities(idx, table_state, variant)
                        .as_bits();
                    possibilities != 0
                        && (possibilities & !playable_mask) == 0
                        && (possibilities & connecting_bit) != 0
                })
            })
        })
    }
}

impl ClueTech for DelayedPlayClue {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                clues_for_player_with_focus(target, pov)
                    .into_iter()
                    .filter_map(|(action, focus_idx)| {
                        let card_id = pov.card_identity(focus_idx)?;
                        if Self::is_delayed_play_situation(card_id, pov) {
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
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        let Some(game_state_snapshot) = history.get(turn.saturating_sub(1)) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());
        let Some(focus) = get_clue_focus(player_index, touched, &giver_pov) else {
            return false;
        };
        // Match if any focus identity consistent with the observer's empathy and the clue mask
        // would have constituted a delayed play from the giver's POV. For non-receiver observers
        // the empathy is a singleton (they see the focus); for the receiver it is wider, and the
        // existential captures her ambiguity over her own card.
        let static_data = observer_pov.static_data();
        let total_ids =
            static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let candidates = observer_pov.inferred_identities(focus).as_bits() & clue_mask;
        (0..total_ids).any(|id| {
            (candidates & (1u64 << id)) != 0 && Self::is_delayed_play_situation(id, &giver_pov)
        })
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        let multi = self.clue_knowledge_updates_multi(
            player_index,
            touched,
            clue,
            turn,
            history,
            observer_pov,
        );
        let mut focus_card: Option<CardDeckIndex> = None;
        let mut mask: u64 = 0;
        for h in &multi {
            for u in &h.immediate {
                if let KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index,
                    mask: m,
                } = u
                {
                    focus_card = Some(*card_deck_index);
                    mask |= m;
                }
            }
        }
        let Some(focus) = focus_card else {
            return Hypothesis::empty();
        };
        Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
            card_deck_index: focus,
            mask,
        }])
    }

    /// Multi-hypothesis variant: when the connecting card for a 1-away focus has
    /// ambiguous-known-playable empathy (e.g. a touched card known to be "some 1"
    /// of unknown color), emit one sub-hypothesis **per candidate connecting
    /// identity**, each pinning the focus to its matching `connecting_id + 1` and
    /// gated on the connecting card playing as that specific identity.
    ///
    /// Sub-hypotheses for the same connecting card share an `alt_group` so that
    /// confirmation of one prunes only the others in that group — sibling techs'
    /// interpretations (e.g. `DirectPlayClue`'s mask on the focus) survive.
    ///
    /// Falls back to the union-mask single-hypothesis behavior for focus
    /// identities whose connecting card is uniquely known to its holder (no
    /// alt_group structure needed) or whose `away_value > 1` (deeper chains —
    /// not yet handled).
    fn clue_knowledge_updates_multi(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> HypothesisSet {
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
            return HypothesisSet::new();
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let focus = match get_clue_focus(player_index, touched, &giver_pov) {
            Some(f) => f,
            None => return HypothesisSet::new(),
        };

        let static_data = observer_pov.static_data();
        let variant = &static_data.variant;
        let total_ids = variant.number_of_suits as usize * variant.stacks_size as usize;
        let clue_mask = variant.empathy_for_clue(clue).as_bits();
        let observer_focus_empathy = observer_pov.inferred_identities(focus).as_bits();
        let candidates = observer_focus_empathy & clue_mask;
        let table_state = giver_pov.table_state();
        let playable_mask = table_state.playable_cards(static_data);
        let num_players = static_data.number_of_players as usize;

        let mut out = HypothesisSet::new();
        // `union_fallback_mask` accumulates focus ids whose connecting card is
        // uniquely known to its holder (no need for an identity-keyed trigger) or
        // whose away_value > 1 (chain refinement not yet implemented). These fall
        // back to a single unconditional Hypothesis carrying their union mask.
        let mut union_fallback_mask: u64 = 0;

        for focus_id in 0..total_ids {
            if candidates & (1u64 << focus_id) == 0 {
                continue;
            }
            let Some(away_value) = giver_pov.away_value(focus_id) else {
                continue;
            };
            if away_value == 0 {
                continue;
            }
            // Note: do NOT apply `is_gotten(focus_id)` here. From the giver's POV
            // the focus card is touched and seen, so its identity always shows up
            // in `gotten_cards` — filtering on it would mask out the very identity
            // we are interpreting from the receiver's POV. `is_gotten` is the right
            // gate for *clue generation* (`is_delayed_play_situation`), not for
            // interpretation.
            if away_value > 1 {
                // Deeper chain — keep the legacy union-mask treatment.
                if Self::connecting_cards_are_known(focus_id, away_value, &giver_pov) {
                    union_fallback_mask |= 1u64 << focus_id;
                }
                continue;
            }
            let connecting_id = focus_id - 1;
            let connecting_bit = 1u64 << connecting_id;

            // Walk each player's hand looking for valid connecting cards for this
            // `connecting_id`.
            for holder in 0..num_players {
                let pk = giver_pov.team_knowledge().player(holder);
                for &idx in table_state.hands[holder].cards() {
                    if !giver_pov.is_touched(idx) {
                        continue;
                    }
                    let strict_match = giver_pov.is_identity_known_to_holder(idx)
                        && giver_pov.card_identity(idx) == Some(connecting_id);
                    let possibilities = pk
                        .combined_possible_identities(idx, table_state, variant)
                        .as_bits();
                    let ambiguous_match = possibilities != 0
                        && (possibilities & !playable_mask) == 0
                        && (possibilities & connecting_bit) != 0;
                    if !(strict_match || ambiguous_match) {
                        continue;
                    }
                    if strict_match {
                        // Holder knows the exact identity — connecting card will
                        // definitely play as `connecting_id`, no need for the
                        // identity-keyed branching. Treat as a fallback union
                        // contribution.
                        union_fallback_mask |= 1u64 << focus_id;
                        continue;
                    }
                    // Ambiguous case: emit a provisional sub-hypothesis keyed on
                    // the connecting card's identity. `alt_group = idx` groups all
                    // sub-hypotheses for the same connecting card so that
                    // confirmation of one prunes its same-card siblings without
                    // touching DirectPlayClue's interpretation in the cohort.
                    let alt_group: AltGroupKey = AltGroupKey::from(idx);
                    out.push(Hypothesis::provisional_grouped(
                        vec![KnowledgeUpdate::NarrowPossibilities {
                            card_deck_index: focus,
                            mask: 1u64 << focus_id,
                        }],
                        PendingTrigger::BlindPlay {
                            player: holder,
                            expected_card: idx,
                            expected_identity: Some(connecting_id),
                        },
                        alt_group,
                    ));
                }
            }
        }

        // Suppress duplicate provisional sub-hypotheses that target the same
        // (focus_id, connecting_card_idx). This happens when iterating
        // `connecting_id` across many candidates that all live in the same
        // holder's empathy — we want one per (alt_group, focus_id) pair.
        // Conservatively dedup on (alt_group, focus_mask, expected_identity).
        // O(n²) on a SmallVec — intentional; `out` is bounded by hand size * candidates.
        let mut seen: smallvec::SmallVec<[(Option<AltGroupKey>, u64, Option<VariantCardId>); 4]> =
            smallvec![];
        out.retain(|h| {
            let key = (
                h.alt_group,
                h.immediate
                    .iter()
                    .find_map(|u| match u {
                        KnowledgeUpdate::NarrowPossibilities { mask, .. } => Some(*mask),
                        _ => None,
                    })
                    .unwrap_or(0),
                h.trigger.as_ref().and_then(|t| match t {
                    PendingTrigger::BlindPlay {
                        expected_identity, ..
                    } => *expected_identity,
                }),
            );
            if seen.iter().any(|k| k == &key) {
                false
            } else {
                seen.push(key);
                true
            }
        });

        if union_fallback_mask != 0 {
            out.push(Hypothesis::unconditional(vec![
                KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index: focus,
                    mask: union_fallback_mask,
                },
            ]));
        }

        out
    }
}

impl HGroupClueTech for DelayedPlayClue {}
impl PlayClueTech for DelayedPlayClue {}
impl_convention_tech_for_hgroup_clue_tech!(DelayedPlayClue, priority::SIMPLE_PLAY_CLUE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn game_actions_returns_empty_when_no_connecting_card_is_visible() {
        // Player 1 has R3 (2-away). Connecting card R2 is not visible anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_returns_empty_when_focus_is_immediately_playable() {
        // Player 1 has R1 (away=0). DirectPlayClue handles this; DelayedPlayClue should skip it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_generates_clue_when_connecting_card_is_visible_in_teammate_hand() {
        // R1 is played on the stack. Player 2 has R2 (card 20, touched+known). Player 1 has R3
        // (card 10). R3 is 1-away; connecting card R2 is touched and known to its holder, so
        // R3 is a valid delayed play clue target.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Player 2 holds deck 20 and knows its identity (clued)
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).visible_cards |= 1 << 20;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().any(|a| matches!(
            a,
            GameAction::Clue {
                player_index: 1,
                ..
            }
        )));
        // Player 2's R2 is directly playable (R1 on stack), so it is NOT a delayed play clue target.
        assert!(actions.iter().all(|a| !matches!(
            a,
            GameAction::Clue {
                player_index: 2,
                ..
            }
        )));
    }

    /// The receiver-friendly path in `connecting_cards_are_known` must NOT fire for
    /// the active player's own cards.  The clue giver works from inferred possibilities
    /// on their own hand, so seeing P2 *among* {R2, Y2, P2} does not guarantee the
    /// card IS P2 — using it as a connector for P3 would be a misidentification.
    #[test]
    fn game_actions_does_not_use_own_ambiguous_card_as_connector() {
        // R1, Y1, P1 played → R2, Y2, P2 all immediately playable.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::{P1, R1, Y1};
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            Y1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            P1.as_variant_card_id(),
            &static_data,
        );

        // p=0 draws deck[10] and it is touched; from p=0's perspective it could be R2, Y2, or P2.
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        // p=1 draws deck[20] = P3 (true identity, visible to the giver).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20);
        table_state.active_player_index = 0; // p=0 is the active clue giver

        // Giver sees P3 in p=1's hand. Their own deck[10] is ambiguous: {R2, Y2, P2}.
        let knowledge = knowledge_with_visible(0, &[(20, P3_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R2_MASK | Y2_MASK | P2_MASK));
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // P3 is 1-away; the only candidate connector is P2 — in p=0's OWN hand,
        // ambiguously. The fix must suppress the delayed play proposal entirely.
        assert!(
            DelayedPlayClue.game_actions(&pov).is_empty(),
            "must not propose a delayed play clue when the only connector is the giver's own \
             ambiguous card"
        );
    }

    /// When the active player has collapsed their own card to a singleton identity
    /// (e.g. via hypothesis narrowing) the strict path of `connecting_cards_are_known`
    /// still fires and the delayed play clue is correctly proposed.
    #[test]
    fn game_actions_uses_own_exactly_known_card_as_connector() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::{P1, R1, Y1};
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            Y1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            P1.as_variant_card_id(),
            &static_data,
        );

        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20);
        table_state.active_player_index = 0;

        // Giver knows their own deck[10] is exactly P2 (singleton empathy).
        let mut knowledge = knowledge_with_visible(0, &[(20, P3_MASK)]);
        knowledge.own_hand |= 1 << 10;
        knowledge.inferred_identities[10] = Some(CardIdentityMask::from_bits(P2_MASK));
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(P2_MASK));
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // P3 is 1-away; connecting card is P2, which p=0 knows exactly. Delayed play
        // clue targeting p=1 must be proposed.
        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(
            actions.iter().any(|a| matches!(
                a,
                GameAction::Clue {
                    player_index: 1,
                    ..
                }
            )),
            "must propose a delayed play clue to p=1 when giver's own card is exactly-known P2"
        );
    }

    /// The receiver-friendly path must NOT use a teammate's card as a connector when the
    /// giver can see its actual identity and it differs from the connecting card — even if
    /// the holder's empathy includes the connecting identity as a possibility.
    #[test]
    fn game_actions_does_not_use_misidentified_teammate_card_as_connector() {
        // Stacks empty → all rank-1 cards are playable.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();

        // p=1 has B2 at deck[10] — potential delayed play target.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);

        // p=2 has a rank-1 clued card at deck[20]; from p=2's perspective it could be
        // any of {R1,Y1,G1,B1,P1} — including B1 (the needed connector for B2). But the
        // giver (p=0) can see it is actually R1, not B1.
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20);
        table_state.clue_touched_cards |= 1 << 20;
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, B2_MASK), (20, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).inferred_identities[20] = Some(CardIdentityMask::from_bits(
            R1_MASK | Y1_MASK | G1_MASK | B1_MASK | P1_MASK,
        ));
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // B2 is 1-away; the only connector candidate is deck[20] which p=2 thinks could be B1,
        // but the giver knows is actually R1. The delayed play must not be proposed.
        assert!(
            DelayedPlayClue.game_actions(&pov).is_empty(),
            "must not propose a delayed play clue when the giver sees the connector candidate \
             is actually a different card"
        );
    }

    #[test]
    fn game_actions_does_not_clue_own_player() {
        // R1 on the stack, player 0 has R3 (own hand), player 1 has R2 (touched + known to holder).
        // All conditions for a delayed play clue on R3 are satisfied, yet no clue targeting
        // player 0 is ever generated.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10); // R3 in player 0's own hand
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20); // R2 in player 1's hand
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= 1 << 20;
        team_knowledge.player_mut(1).visible_cards |= 1 << 20;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().all(|a| !matches!(
            a,
            GameAction::Clue {
                player_index: 0,
                ..
            }
        )));
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_focus_is_delayed_playable_with_connecting_card_visible() {
        // R1 is played on the stack. R3 (1-away) focus, R2 (connecting card) touched+known.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 20;
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).visible_cards |= 1 << 20;

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            }, // red clue
            turn: 1,
        };
        assert!(DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_immediately_playable() {
        // R1 has away=0, so it's not a delayed play clue.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Rank,
                clue_value: 1,
            },
            turn: 1,
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_connecting_card_not_visible() {
        // R3 is 2-away but no connecting card (R2) is touched or known anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1,
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!DelayedPlayClue.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 1
            },
            &[],
            &pov
        ));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_narrows_focus_to_delayed_playable_ids() {
        // R1 is played on the stack. Player 0 (receiver) has R3 (card 10, touched).
        // Player 1 has R2 (card 20, visible) → R3 is a valid delayed play target (away=1, R2 visible).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10); // R3 in player 0's hand
        table_state.clue_touched_cards |= 1 << 10;
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20); // R2 in player 1's hand
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0; // Clue giver

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 20;
        team_knowledge.player_mut(1).own_hand |= 1 << 20;
        team_knowledge.player_mut(1).visible_cards |= 1 << 20;

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = DelayedPlayClue.knowledge_updates(
            &GameAction::Clue {
                player_index: 0,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 1,
            },
            &[snapshot],
            &pov,
        );

        assert_eq!(updates.immediate.len(), 1);
        assert!(updates.trigger.is_none());
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates.immediate[0]
        {
            assert_eq!(*card_deck_index, 10);
            // R3 (id=2) must be in the mask; R1 (id=0, away=0) and R2 (id=1, away=0 after R1 played)
            // must not be.
            assert_ne!(mask & R3_MASK, 0, "R3 should be in the mask");
            assert_eq!(mask & R1_MASK, 0, "R1 (played) should not be in the mask");
            assert_eq!(
                mask & R2_MASK,
                0,
                "R2 (immediately playable) should not be in the mask"
            );
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        // get_clue_focus returns None for an empty touched list → no update.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            DelayedPlayClue
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1
                    },
                    &[snapshot],
                    &pov
                )
                .is_empty()
        );
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_delayed_playable_ids_exist() {
        // All stacks complete → every card has away=None (already played) → mask is 0 → no update.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
        for &card_id in &[
            R1, R2, R3, R4, R5, Y1, Y2, Y3, Y4, Y5, G1, G2, G3, G4, G5, B1, B2, B3, B4, B5, P1, P2,
            P3, P4, P5,
        ] {
            table_state.update_with_play_action_of_specific_card(0, card_id as usize, &static_data);
        }
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            DelayedPlayClue
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 1,
                        touched_card_deck_indexes: smallvec::smallvec![10],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1
                    },
                    &[snapshot],
                    &pov
                )
                .is_empty()
        );
    }

    #[test]
    fn knowledge_updates_recognizes_delayed_play_via_ambiguous_known_playable_one() {
        // B1 and P1 are played. Player 2 holds card 30, touched, with empathy narrowed
        // to {R1, Y1, G1} (i.e. "known playable 1 of unknown color" — typical good-touch
        // result of a rank-1 clue after B1/P1 are off the table). Player 0 then clues
        // rank 2 to player 1, focusing card 40.
        //
        // The receiver-friendly path of `connecting_cards_are_known` should recognise
        // that card 30 is a valid connecting card for any of R2/Y2/G2, so the focus
        // mask returned by `knowledge_updates` must include R2, Y2, and G2.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::{B1, P1};
        table_state.update_with_play_action_of_specific_card(
            0,
            B1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            P1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(40); // R2 in player 1's hand (focus)
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(30); // R1 in player 2's hand (connecting)
        table_state.clue_touched_cards |= 1 << 30;
        table_state.active_player_index = 0; // Clue giver

        let known_playable_one_mask = R1_MASK | Y1_MASK | G1_MASK;
        let rank2_mask = R2_MASK | Y2_MASK | G2_MASK | B2_MASK | P2_MASK;

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Player 0 (giver) sees both cards' true identities.
        team_knowledge.player_mut(0).inferred_identities[40] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 40;
        team_knowledge.player_mut(0).inferred_identities[30] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 30;
        // Player 2 holds card 30; their empathy narrows to the three remaining 1s.
        team_knowledge.player_mut(2).own_hand |= 1 << 30;
        team_knowledge.player_mut(2).inferred_identities[30] =
            Some(CardIdentityMask::from_bits(known_playable_one_mask));
        // Player 1 (receiver) holds card 40; after the rank-2 clue their empathy is the full rank-2 mask.
        team_knowledge.player_mut(1).own_hand |= 1u64 << 40;
        team_knowledge.player_mut(1).inferred_identities[40] =
            Some(CardIdentityMask::from_bits(rank2_mask));

        // Snapshot: active player is the giver (player 0).
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        // Observer is the receiver (player 1); update active_player accordingly.
        table_state.active_player_index = 1;
        let mut p1_knowledge = PlayerKnowledge::new(1);
        p1_knowledge.own_hand = 1u64 << 40;
        p1_knowledge.inferred_identities[40] = Some(CardIdentityMask::from_bits(rank2_mask));
        let pov = LightweightPlayerPOV::new(
            1,
            &p1_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

        let updates = DelayedPlayClue.knowledge_updates(
            &GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec::smallvec![40],
                clue: Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 2,
                },
                turn: 1,
            },
            &[snapshot],
            &pov,
        );

        assert_eq!(updates.immediate.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates.immediate[0]
        {
            assert_eq!(*card_deck_index, 40);
            assert_ne!(mask & R2_MASK, 0, "R2 should be in the delayed-play mask");
            assert_ne!(mask & Y2_MASK, 0, "Y2 should be in the delayed-play mask");
            assert_ne!(mask & G2_MASK, 0, "G2 should be in the delayed-play mask");
            // B2 and P2 are already directly playable (away=0), so they are not
            // delayed — DelayedPlayClue must not include them.
            assert_eq!(
                mask & B2_MASK,
                0,
                "B2 (directly playable) must not be in delayed mask"
            );
            assert_eq!(
                mask & P2_MASK,
                0,
                "P2 (directly playable) must not be in delayed mask"
            );
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    /// `clue_knowledge_updates_multi` must split the delayed-play interpretation
    /// across each candidate connecting identity when the connecting card is
    /// ambiguous-known-playable in its holder's empathy. Each sub-hypothesis
    /// pins the focus to `connecting_id + 1`, carries a BlindPlay trigger keyed
    /// on the connecting card's deck index AND its identity, and shares an
    /// `alt_group` with its same-card siblings.
    #[test]
    fn clue_knowledge_updates_multi_emits_per_connecting_id_subhypotheses() {
        use crate::engine::convention::convention_tech::ClueTech;
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::{B1, P1};
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // B1 and P1 already played → playable_cards = {R1, Y1, G1, B2, P2}.
        table_state.update_with_play_action_of_specific_card(
            0,
            B1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            0,
            P1.as_variant_card_id(),
            &static_data,
        );
        // Receiver (player 0) holds the focus card (deck 30) and the connecting
        // card (deck 10). Connecting card is touched.
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(30); // R2 (focus)
        table_state.update_with_draw_action(10); // R1 (connecting, will be touched)
        table_state.clue_touched_cards |= 1 << 10;
        // Clue giver is player 1.
        table_state.active_player_index = 1;

        let known_playable_one_mask = R1_MASK | Y1_MASK | G1_MASK;

        // Player 0's knowledge: connecting card has empathy {R1, Y1, G1}.
        let mut p0_knowledge = PlayerKnowledge::new(0);
        p0_knowledge.own_hand = (1u64 << 30) | (1u64 << 10);
        p0_knowledge.inferred_identities[10] =
            Some(CardIdentityMask::from_bits(known_playable_one_mask));

        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = (1u64 << 30) | (1u64 << 10);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(known_playable_one_mask));

        // Snapshot represents the post-raw-clue state (which gives the receiver
        // the rank-2 narrowing on the focus). For test simplicity, set the
        // receiver's empathy on focus to the full rank-2 mask.
        let rank2_mask = R2_MASK | Y2_MASK | G2_MASK | B2_MASK | P2_MASK;
        p0_knowledge.inferred_identities[30] = Some(CardIdentityMask::from_bits(rank2_mask));
        team_knowledge.player_mut(0).inferred_identities[30] =
            Some(CardIdentityMask::from_bits(rank2_mask));

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        // Observer is the receiver (player 0).
        let pov = LightweightPlayerPOV::new(
            0,
            &p0_knowledge,
            &team_knowledge,
            &table_state,
            &static_data,
        );

        let updates = DelayedPlayClue.clue_knowledge_updates_multi(
            0,
            &[30],
            &Clue {
                clue_type: ClueType::Rank,
                clue_value: 2,
            },
            0,
            &[snapshot],
            &pov,
        );

        // Expect exactly three per-connecting-id sub-hypotheses (R1, Y1, G1).
        // Each pins the focus to a single rank-2 identity and carries an
        // identity-keyed trigger.
        assert_eq!(
            updates.len(),
            3,
            "expected three identity-keyed sub-hypotheses (one per candidate connecting id)"
        );
        let mut got_masks: Vec<u64> = Vec::new();
        let mut got_alt_groups: Vec<Option<AltGroupKey>> = Vec::new();
        for h in &updates {
            assert_eq!(h.immediate.len(), 1);
            match &h.immediate[0] {
                KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index,
                    mask,
                } => {
                    assert_eq!(*card_deck_index, 30);
                    got_masks.push(*mask);
                }
                _ => panic!("expected NarrowPossibilities"),
            }
            match &h.trigger {
                Some(PendingTrigger::BlindPlay {
                    player,
                    expected_card,
                    expected_identity,
                    ..
                }) => {
                    assert_eq!(*player, 0, "trigger should fire on player 0 playing");
                    assert_eq!(
                        *expected_card, 10,
                        "trigger should key on connecting deck idx"
                    );
                    assert!(
                        expected_identity.is_some(),
                        "trigger must be identity-keyed for selective rejection"
                    );
                }
                _ => panic!("expected BlindPlay trigger"),
            }
            got_alt_groups.push(h.alt_group);
        }
        got_masks.sort();
        let mut expected_masks = vec![R2_MASK, Y2_MASK, G2_MASK];
        expected_masks.sort();
        assert_eq!(got_masks, expected_masks);
        // All sub-hypotheses for the same connecting card share an alt_group.
        assert!(
            got_alt_groups.iter().all(|g| *g == Some(10)),
            "all sub-hypotheses must share alt_group keyed on connecting card index"
        );
    }
}
