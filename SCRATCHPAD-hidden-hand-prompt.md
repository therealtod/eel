# Scratchpad: prompt interpretation fails when the connecting card is in the bot's own (hidden) hand

Status: **Stage 1 DONE (static fix landed + verified). Stage 2 (sliding promise) and the
SimpleFinesse audit are still open.** This note hands the remaining work to the next agent/dev.

## ✅ What has already been done (Stage 1)

The static empathy-based fix is **implemented, tested, and the full suite is green**
(378 unit + 53 integration tests pass; `should_not_dupe_own_y2`,
`should_not_dupe_own_y2_in_liveplay`, and `should_choose_the_most_efficient_play_clue` all
pass together).

The fix went a different (cleaner) route than the "no visible teammate connector" guard
originally sketched below: it gates the empathy-trust fallback on whether the clue is a
**real (grounded)** clue vs. a **hypothetical clue projected inside search**.

**Why a grounded gate and not just the visibility guard:** the efficiency fixture makes
*all* of Alice's cards `"id":"x"`, so neither "singleton empathy" nor "giver can't see any of
target's hand" discriminates the live prompt from the search phantom — both collapse. The
only robust discriminator is real-clue vs. hypothetical-search-clue. During search the inner
giver provably could not have seen the root's hand (`effective_visible` excludes it), so it
could not have intended a prompt onto the root's own hidden card; trusting one there
fabricates a phantom self-prompt chain. A real partner's clue *does* warrant that trust.

**Changes landed:**
- `src/engine/knowledge/player_pov.rs` — new defaulted trait method `fn is_grounded(&self) ->
  bool { false }` on `PlayerPOV`.
- `src/engine/knowledge/lightweight_player_pov.rs` — `grounded: bool` field (defaults false),
  `with_grounded(bool)` builder, `is_grounded()` impl.
- `src/engine/knowledge_aware_game_state.rs` — `apply` passes `resolver.advances_real_game()`
  into `apply_clue`; `apply_clue`, `interpret_clue_for_receiver`, and
  `interpret_clue_for_observers` thread `grounded` through and build their POVs with
  `.with_grounded(grounded)`.
- `src/engine/convention/hgroup/tech/simple_prompt.rs` — both the `matches_clue` empathy
  fallback and the `clue_knowledge_updates` **Case 1** `.or_else(...)` fallback are gated on
  `observer_pov.is_grounded()` (Case 1 early-returns `None` when not grounded). The "no
  visible teammate holds the connector" guard (`teammate_holds_connector`) is retained.
- `simple_prompt.rs` tests — added `hidden_own_connector_case1_setup()` helper plus
  `case1_narrows_own_hidden_connector_for_grounded_clue` (asserts `NarrowPossibilities {
  card_deck_index: 5, mask: R2_MASK }`) and `case1_does_not_narrow_own_hidden_connector_in_search`
  (asserts no updates). These are the regression the omniscient path structurally cannot catch.

**Skip rule still relies on the leftmost-eligible card** — see Stage 2 below for the sliding
case that the static `NarrowPossibilities` does not yet handle.

---

## Original diagnosis (kept for reference)

## TL;DR

The H-Group **prompt** convention never actually fires when the promised connecting card is
in the *interpreting bot's own hand*. The prompt resolution relies on the reconstructed
**giver** knowing the connector's true identity (`card_identity`), which is impossible in
live play because the bot cannot see its own hand. The omniscient replay reconstruction
hides this bug by revealing every card, so the bot "knows" its card for free and the prompt
machinery is never exercised.

## Reproduction

```bash
cargo test --test '*' should_not_dupe_own_y2 -- --nocapture
```

- `should_not_dupe_own_y2`            (omniscient, `engine_action_at_turn`)            → **passes**
- `should_not_dupe_own_y2_in_liveplay` (hidden seat, `engine_action_at_turn_hidden_seat`) → **fails**

Both live in `tests/integration/replay_regression.rs` (~line 592 and 605). Both expect Bob
(player 1) to `Play { card_deck_index: 7 }` (his Y2). In hidden mode Bob instead gives a
`Color 1` clue to Alice — he never realizes deck[7] is Y2.

### The scenario (fixture `should_not_dupe_own_y2/replay.json`, turn 11)

- Seats: 0 = Alice, 1 = Bob, 2 = Cathy. Bob is on turn (active) at turn 11.
- Y1 is already played (yellow stack height 1).
- Alice just clued **yellow to Cathy**, focus = **Y3** at deck[18].
- Bob holds two touched 2s:
  - deck[8] = **R2**, already locked to R2 (singleton empathy `{R2}`).
  - deck[7] = **Y2**, touched as a rank-2 clue, empathy `{R2,Y2,G2,B2}`.
- Correct H-Group read: Y3 needs Y2 → **Prompt**. Bob's leftmost touched card whose empathy
  admits Y2 is deck[7] (deck[8] is skipped because its empathy excludes Y2). So Bob's
  deck[7] is promised Y2; Bob should play it and must **not** clue/duplicate Alice's Y2.

## Root cause (precise)

Decisive probe (added to a throwaway `tests/_diag.rs`; gitignored) — reconstruct the
**giver Alice's** POV just before her clue and ask what she sees of Bob's connector:

| reconstruction | `alice.card_identity(7)` |
|---|---|
| `from_hanablive` (omniscient) | `Some(6)` = Y2 |
| `from_hanablive_hidden_seat(hidden=1)` (live) | **`None`** |

Chain of blame:

1. **The hidden-seat reconstruction is correct, not the bug.**
   `ReplayRunner::from_hanablive_hidden_seat(game, conv, hidden=Bob)` hides Bob's hand from
   *every* player's view, including Alice's reconstructed POV. That faithfully models live
   play: the live bot *is* Bob and cannot know his own card identities, so he also cannot
   reconstruct what Alice "sees" in his hand.

2. **The prompt interpretation depends on ground-truth sight the live bot lacks.**
   - `src/engine/convention/hgroup/tech/simple_prompt.rs`, `clue_knowledge_updates`,
     **Case 1** (current player is the prompted player), line ~179:
     gates on `Self::is_valid_prompt_situation(connecting_id, current, focus, &giver_pov)`.
   - `is_valid_prompt_situation` → `prompt_connector_index`
     (`src/engine/convention/hgroup/h_group_core/hand_position.rs:114-116`) requires
     **`pov.card_identity(idx) == Some(connecting_id)`** — i.e. the reconstructed giver must
     *know* deck[7] is Y2.
   - In hidden mode `giver_pov.card_identity(7) == None` → gate fails → `Hypothesis::empty()`
     → no prompt hypothesis → Bob's deck[7] empathy stays `{R2,Y2,G2,B2}`.

3. **Omniscient masks it.** There Alice's POV sees deck[7]=Y2 so the gate passes; *and*
   independently the full-deck reveal collapses Bob's deck[7] empathy to Y2 directly. Either
   way Bob "knows" Y2 and plays it — on information the live bot never has. Note: in **both**
   modes the dumped `hypotheses` count is **0** at the decision point — confirming the prompt
   convention never fires; omniscient just gets the answer from the card reveal.

### Already-correct ingredient

The same function already finds the connector **by empathy** at
`simple_prompt.rs:187-194` (iterates Bob's touched cards, picks the first whose
`combined_possible_identities` admits `connecting_id`). That empathy *is* available in
hidden mode (the snapshot's `team_knowledge.player(Bob)` carries deck[7] = `{R2,Y2,G2,B2}`).
**Only the identity-based validity gate at line 179 blocks it.**

## Fix direction

A player interpreting a prompt that lands on **their own hidden hand** must identify the
connector by **empathy + the H-Group trust convention**, not by the giver's ground-truth
sight. The line-179 identity gate is correct for *clue generation* (`clue_game_actions`) and
for *spectator* interpretation, but wrong for self-interpretation of a hidden hand.

Do **not** simply delete the gate. Two correctness constraints must be preserved:

1. **Skip rule.** The connector is the leftmost touched card whose empathy *admits*
   `connecting_id`; cards whose empathy *excludes* it (e.g. deck[8] locked to R2) are skipped.
   The chain variant (intervening cards all currently playable) must also still hold. See the
   skip/chain logic in `prompt_connector_index`.
2. **Don't steal a visible teammate's prompt.** Bob must claim the prompt for his own hand
   only when no *other* non-giver player **visibly** holds the connector (a card Bob can see
   to be `connecting_id`). If a teammate visibly holds the clued connector, the prompt is
   theirs and Bob must emit nothing.

Concretely: in Case 1, replace the identity gate with an empathy-based connector selection
on `current`'s own hand, guarded by the "no visible teammate connector" check. The existing
line-187 `find` is close but currently unguarded and doesn't encode the skip/chain rule —
prefer routing through a connector-finder that takes empathy for the *target's own* hand and
truth for *other* hands.

### ⚠️ The hard part: the prompt is a *sliding* promise, not a one-shot narrow

(Raised by the user — must be designed for, even if the current fixture doesn't exercise it.)

The promise is on the **identity**, attached **provisionally** to the current leftmost
eligible touched card. If that card is played and revealed to be a *different* (still
consistent) identity, the **promise slides right to the next eligible touched card.**

Worked example (a variant of the fixture):
- Suppose deck[8] (slot 3) had **not** been locked to R2 — its empathy still admits Y2.
- Then Bob attributes Y2 to deck[8] (now the leftmost eligible) and plays it.
- It reveals **R2**. Bob must understand: Y2 is *still promised*, and the promised card is
  now the next eligible slot (deck[7], slot 4). He plays that next.

Implication for the implementation: a static
`NarrowPossibilities { card_deck_index: <fixed>, mask: Y2 }` is **insufficient in general**.
It happens to work for the current fixture *only because deck[8] is already locked to R2*, so
the leftmost-eligible card is unambiguously deck[7].

For the sliding behavior you need one of:
- **(A) Re-resolving inference.** Model the prompt as "identity Y2 is promised to player P";
  each turn, resolve the promised card as the current leftmost touched card of P whose
  empathy admits Y2. After a play reveals truth and a stack/empathy update lands, the
  resolution naturally moves to the next eligible card. This is the cleanest semantic match
  but doesn't fit the current one-shot `Hypothesis`/`NarrowPossibilities` model directly.
- **(B) Trigger-driven re-emit.** Use a provisional hypothesis with a
  `PendingTrigger::BlindPlay { player, expected_card, expected_identity: Some(Y2) }`
  (`src/engine/knowledge/knowledge_update.rs:165`). Today an identity **mismatch only
  *rejects* (drops)** the hypothesis — it does **not** re-attach the promise to the next
  card. You'd need to extend resolution so a "played but revealed a different consistent
  identity" outcome **re-emits** the promise onto the next eligible touched card, rather than
  silently dropping it.

Relevant existing primitives to study before choosing:
- `Signal::Play { card_deck_index, committed_identity }`
  (`src/engine/convention/hgroup/signal.rs`) — commits a *specific* card to an identity; a
  reveal that contradicts `committed_identity` is not currently "slid", and
  `neutralize_landed_play_signals` (in `knowledge_aware_game_state.rs`) only retires signals
  whose identity has *landed on the stacks*, not signals whose card revealed a different id.
- `Hypothesis` / `PendingTrigger` / `TrackedHypothesis`
  (`src/engine/knowledge/knowledge_update.rs`) — cohort/tier/trigger machinery.
- `KnowledgeAwareGameState::apply_clue` → `interpret_clue_for_observers`
  (`src/engine/knowledge_aware_game_state.rs:876`). NB: this builds the observer's POV with
  `self.team_knowledge.player(giver)` as the knowledge but the *target's* index (line ~893);
  SimplePrompt re-derives `giver_pov` from the snapshot internally and only uses
  `observer_pov.player_index()`, so this oddity is not the root cause — but verify it when
  threading own-hand empathy through.

Recommended staging:
1. ~~Land the **static** empathy-based fix so `should_not_dupe_own_y2_in_liveplay` passes; add
   a Case-1 hidden-connector unit test.~~ **✅ DONE** — see "What has already been done" at the
   top. (Implemented via the grounded gate rather than the visibility guard; rationale there.)
2. **STILL OPEN** — design the **sliding** mechanism (A or B above) and add a regression
   fixture/scenario covering the "leftmost eligible reveals a different identity → promise
   slides" case. The current static `NarrowPossibilities` works for the fixture only because
   deck[8] is already locked to R2.

## Other things to check (same blind spot likely elsewhere)

- **`SimpleFinesse`** (`tech/simple_finesse.rs`) almost certainly has the same hidden-own-hand
  dependency on the giver's `card_identity` for the finesse-position card. Audit it for the
  same divergence (finesse onto the interpreting bot's own finesse position).
- Any other tech that calls `prompt_connector_index` / `has_on_finesse_position` /
  `card_identity` while interpreting an action whose promised card can be in the observer's
  own hand.

## Verify after fixing

```bash
cargo test --test '*' should_not_dupe_own_y2          # both variants must pass
cargo test                                            # full suite — watch prompt/finesse tests
```

Throwaway diagnostics live in `tests/_diag.rs` (gitignored); `diag_dupe_compare` there dumps
the full/hidden before/after empathy and the Alice-POV `card_identity` probe used above.
Delete or keep — it never commits.
