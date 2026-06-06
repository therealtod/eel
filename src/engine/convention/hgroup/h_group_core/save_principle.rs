use super::get_chop_index;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// True when `card_id` could be the focus of *some* H-Group play clue tech in the current
/// position — i.e. the team could clue it to be played, now or via a connecting chain. This
/// unifies the four play clue techs by the precondition they all share:
///
/// - **`DirectPlayClue`** — the identity is immediately playable on the stacks (`away_value == 0`).
/// - **`DelayedPlayClue` / `SimplePrompt`** — the focus is `n`-away and the connecting cards down to
///   the stack are present (clued / known) in some hand.
/// - **`SimpleFinesse`** — the focus is 1-away and the connecting card sits unclued on a teammate's
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

/// <https://hanabi.github.io/docs/beginner/#the-save-principle>  (Save Principle)
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
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::deck::unit_test_constants::novariant_constants::{
        R1_MASK, R2_MASK, R3_MASK, R4_MASK,
    };
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── Save Principle (is_save_principle_protected / is_forced_save) ────────

    /// Build a one-card chop for player 1 (receiver) visible to player 0 (observer), optionally
    /// discarding `discard_id` copies first to manipulate criticality. Returns a ready POV plus
    /// the owning `table_state/knowledge` so the caller can keep them alive.
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
