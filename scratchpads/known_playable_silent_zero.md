# `known_playable_in_hands` silently returning 0 at the leaf

## Symptom

Replay `should_play_known_playable.json`, turn 6. Alice holds a touched
`deck[2]` with empathy `{Y1, G1, P1}` — all currently playable. `PlayKnownPlayable`
correctly proposes `Play deck[2]`. The engine instead picks `Discard chop`
(`deck[0]`).

Test: `tests/replay_regression.rs::should_play_known_playable`.

## What the search sees

Top two PVs at the root (Alice, turn 6):

```
total=50.139  Discard chop (deck[0])                   (chosen)
  leaf: game_score=60  known_playable=0.0  crit_exposure=0.0  total=46.139
  [0] DiscardChop      Alice discards deck[0]
  [1] PlayKnownPlayable Bob plays deck[7]
  [2] PlayKnownPlayable Cathy plays deck[14]
  [3] PlayKnownPlayable Alice plays deck[2]
  [4] PlayKnownPlayable Bob plays deck[15]
  [5] DiscardChop

total=40.450  Play deck[2]                             (the move we wanted)
  leaf: game_score=50  known_playable=0.0  crit_exposure=0.0  total=36.454
  [0] PlayKnownPlayable Alice plays deck[2]
  [1] PlayKnownPlayable Bob plays deck[15]
  [2] DelayedPlayClue   Cathy clues color-4 → Alice [3, 0]
  [3] DelayedPlayClue   Alice clues rank-3 → Bob [9]
  [4] PlayKnownPlayable Bob plays deck[7]
  [5] DiscardChop
```

The 9.7-point gap is entirely in the leaf, dominated by a 10-point `game_score`
gap (4 rollout plays vs 3 rollout plays).

## Why this is not the previously-documented bug families

Initial hypothesis: rollout policy asymmetry from `rollout_outsources_disambiguation.md`
(failure mode #1, "asymmetric per-turn action selection across sibling subtrees").

This turned out to be wrong. Investigation:

1. **`clues_for_player_with_focus` returns identical candidate sets** for Bob in
   all sibling subtrees at Alice's depth-3 turn 9 (7 candidates, same focus
   choices). So clue *enumeration* is consistent.

2. **`DelayedPlayClue` filters those candidates through `is_delayed_play_situation`**,
   which itself calls `connecting_cards_are_known`. The proposed clue
   "rank-3 → Bob (focus deck[9] = P3)" requires connectors for both P2 and P1.

3. **The P2 connector exists in subtrees B/C but not in subtree A** — verified by
   instrumenting `connecting_cards_are_known`. In subtree B (Cathy clued
   rank-2 → Alice's deck[0]), Alice's deck[0] empathy collapsed to a singleton
   {P2} (Good Touch + the rank-2 clue mask + visible rank-2s elsewhere). The
   STRICT path matches: `is_identity_known_to_holder(deck[0]) &&
   card_identity == P2`. In subtree A, no clue was given, so no player has a
   card narrowed to P2.

So the rollout's choice is *consistent with the state* — the state genuinely
differs because the team has different information.

Also not draw-dilution (`draw_dilution_bias.md`): leaf-shape terms
(`team_empathy`, `crit_exposure`, `efficiency`) are not load-bearing here.
`team_empathy_bonus=0.0` and `critical_exposure_penalty=0.0` in both leaves.

## What is actually wrong

Two compounding issues that both let the discard line win:

### Issue A — `known_playable_in_hands` returning 0 at the leaf when it shouldn't

At the PLAY line's leaf (turn 12), stacks are R=1, Y=1, G=1, B=1, P=1. Cathy's
`deck[14]` is touched (rank-2 clue from turn 5) with empathy `{R2, Y2, G2}` (or
similar narrowed subset). All three identities are in the current playable mask.

`known_playable_in_hands` should fire Priority 3 (game-rule empathy ⊆ playable):

```rust
let bits = table_state.deck.get_global_empathy(idx).as_bits();
if bits != 0 && (bits & playable_mask) == bits {
    total += 1.0;
}
```

But the leaf breakdown shows `known_playable: 0.0`. Either:
- Phantom-play state at the leaf is producing a different effective
  `playable_mask` than the visible `stacks` would suggest (commit `06a30a4`
  moved phantom_plays to the search layer; this may affect `playable_cards`
  reads downstream), or
- The truth-id early return at the top of `known_playable_in_hands` is filtering
  Cathy's deck[14] because `truth.card_identity(deck[14]) == Some(R2)` and
  the leaf's `playable_mask` does not include R2 (i.e. R-stack has advanced
  to 2 in some phantom-aware way), or
- The `get_global_empathy` path returns 0 / a wider mask than expected.

The eval has the term and weight `1.0`, but it's silent. If it fired correctly
for Cathy's queued deck[14] in the PLAY-line leaf, +1.0 would close ~1 point of
the gap — and probably more since the search would re-rank the inner Cathy-turn-8
choice when the "Cathy plays deck[14]" sub-leaf is also credited with its own
queued plays.

### Issue B — chop discards with wide empathy are free to the search

Alice's `deck[0]` is actually P2 — a near-critical card (2 copies of each
rank-2 in No Variant). From Alice's POV the empathy is wide so she doesn't
know. The leaf's `critical_exposure_penalty = 0.0` in the discard line: the
existing term does not price in the *expected* criticality of a yet-to-be-
discarded card.

Combined with the +2 half-token bonus from discarding, this makes the discard
appear strictly positive in the search's economy. The discard is also "free"
because Alice's queued known-playable (`deck[2]`) gets played later in the
rollout horizon anyway — she pays nothing for deferring it.

## Ordering and plan

Start with Issue A. The fix is local, the silent-zero is concrete, and fixing
it likely closes the regression test without needing the broader chop-risk
work in Issue B.

1. Add a focused diag test: construct the PLAY-line leaf state, call
   `known_playable_in_hands`, assert it is >= 1 (because Cathy's deck[14]
   should qualify).
2. Walk the priority gates to find which one is filtering.
3. Fix and re-run `should_play_known_playable` plus the wider suite.

If Issue A doesn't close the test alone, return to Issue B as a follow-up.
Issue B is the conceptually right structural fix (chop discards should not be
free when empathy is wide enough to plausibly cover criticals) but it touches
the evaluator's penalty shape, which is more invasive.

## What was ruled out / didn't apply

- `critical_exposure_followups.md` #3 (concentration): irrelevant —
  `critical_exposure_penalty = 0` in both leaves.
- `critical_exposure_followups.md` #4 (draw potential): the 10-point game_score
  gap is too large for a small-weight leaf bonus to close.
- `rollout_outsources_disambiguation.md` #A/B/C/D: don't apply — rollout
  policy here is consistent, not asymmetric.
- `draw_dilution_bias.md` empathy normalization: the load-bearing leaf
  difference is `game_score`, not `team_empathy`.
