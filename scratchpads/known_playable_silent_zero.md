# `known_playable_in_hands` silently returning 0 at the leaf

## Symptom

Replay `should_play_known_playable.json`, turn 7. Alice holds a touched
`deck[2]` with empathy `{Y1, G1, P1}` — all currently playable. `PlayKnownPlayable`
correctly proposes `Play deck[2]`. The engine instead picks `Discard chop`
(`deck[0]`).

Test: `tests/replay_regression.rs::should_play_known_playable`.

## What the search sees

Top two PVs at the root (Alice, turn 7):

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
   all sibling subtrees at Alice's depth-3 turn 10 (7 candidates, same focus
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

### Issue A — `known_playable_in_hands` returning 0 at the leaf (investigated, ruled out as primary)

At the PLAY line's leaf the team's empathy on Cathy's `deck[14]` is `{R2, Y2,
G2, B2, P2}` (rank-2 clue, not narrowed by Good Touch because phantom plays do
not reapply it). `known_playable_in_hands` checks `empathy ⊆ playable_mask`.

Cause of the zero: phantom plays remove the card from the hand but **do not
advance the playing stack**. So `table_state.playing_stacks` only reflects
real-identity-resolved plays. At the PLAY-line leaf:

| | DISCARD leaf | PLAY leaf |
|---|---|---|
| real stacks | R=1, G=1, B=1 | R=1, G=1, B=1 |
| phantom plays | 3 | 2 |
| `state.score(variant, phantom)` | 6 → game_score=60 | 5 → game_score=50 |
| `playable_mask` | {R2, Y1, G2, B2, P1} | {R2, Y1, G2, B2, P1} |

Cathy's deck[14] empathy `{R2, Y2, G2, B2, P2}` is not ⊆ `{R2, Y1, G2, B2,
P1}` (Y2, P2 missing) → not counted. **This is the conservative-correct
answer:** with 2 phantom plays into suits ⊆ {Y, G, P}, at most two of those
suits' stacks have advanced; one of {Y2, P2} is genuinely not yet playable in
any consistent extension. The team cannot confidently call this card known-
playable.

**Attempted fix that didn't work:** expanding `playable_mask` by `phantom_plays`
ranks per suit. Let `effective_mask = playable_mask ∪ {(s, r) : r ∈ stack[s]+1
..stack[s]+phantom_plays}`. Result: leaf `known_playable` went up but the
DISCARD line gained 5.0 while the PLAY line gained only 2.0 — the DISCARD line
has MORE phantom plays (3 vs 2), so its effective mask is wider, and its
leaf-hand cards (rank-2-style wide empathy) all gained credit. The fix
**widened the wrong-direction gap from 9.7 to 12.7**.

The expansion is unsound: extending the playable_mask by phantom_plays
overestimates which cards each individual player believes are playable
(phantom plays are shared resources — only ONE suit's stack advances per
phantom, not all of them).

Reverted. The silent zero is correct.

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

**Issue A is investigated and ruled out** as a fixable cause — the silent
zero is conservative-correct given how phantom plays interact with the leaf
playable_mask. A naive "widen the mask by phantom_plays" fix actively
worsens the regression. A correct phantom-aware fix would need per-suit
phantom tracking (which suit's stack each phantom play "would have"
advanced) and isn't a local change to this function.

**Pivot to Issue B.** The actual lever for closing the gap on this test is
the chop-discard pricing. The discard line wins because:

1. Alice's chop discard is "free" — `critical_exposure_penalty=0` on the
   wide-empathy chop, so the eval doesn't see the expected cost of losing
   P2 (a near-critical 2).
2. Discarding adds +2 half-tokens, which fuels the rollout's longer play
   cascade.
3. The phantom-play accounting at the leaf means game_score reflects "total
   plays in the rollout window," which the DISCARD line wins 4-to-3 because
   it doesn't burn a clue token on Alice's turn 7.

A targeted fix: `critical_exposure_penalty` (or a sibling term) should
charge the actor an **expected criticality cost** when discarding a card
whose empathy plausibly contains critical or near-critical cards. The
chop's `get_global_empathy` is wide; the eval can compute
`E[criticality] = Σ_{id ∈ empathy} P(card = id) · criticality(id)` and
charge that as a discard cost.

Even simpler first cut: charge `signal_ignored_penalty`-style for
"discarding while holding a known-playable in own hand" — Alice's deck[2]
is touched and known-playable; discarding deck[0] while deck[2] sits there
deferred-to-the-future should not be free.

The fallback from `draw_dilution_bias.md` ("extend `signal_ignored_penalty`
so it also charges the actor for ignoring a touched known-playable in their
own hand") is the smallest version of this and may close the test on its
own.

## Followup: also look at phantom-play Good-Touch reapply

Separately worth investigating: the `if stack_advanced { reapply_good_touch }`
gate in `apply_play`. Phantom plays do not commit to identity but DO inform
the team that "some card in the played card's empathy is now trash." This
could narrow other touched cards' empathy in the rollout. Not the cause of
the current failure, but a real fidelity loss in the rollout state. File
under "narrow empathy aggressively when phantom plays happen."

## What was ruled out / didn't apply

- `critical_exposure_followups.md` #3 (concentration): irrelevant —
  `critical_exposure_penalty = 0` in both leaves.
- `critical_exposure_followups.md` #4 (draw potential): the 10-point game_score
  gap is too large for a small-weight leaf bonus to close.
- `rollout_outsources_disambiguation.md` #A/B/C/D: don't apply — rollout
  policy here is consistent, not asymmetric.
- `draw_dilution_bias.md` empathy normalization: the load-bearing leaf
  difference is `game_score`, not `team_empathy`.
