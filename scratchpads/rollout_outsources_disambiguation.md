# Rollout-asymmetric outsourcing of own-hand disambiguation

## Symptom

Replay `discards_for_no_good_reason.json`, turn 9. Alice holds:

| slot | actual | empathy | touched |
|---|---|---|---|
| deck[1]  | R5 | any-5                  | yes |
| deck[3]  | P3 | ranks 2–4, not Yellow  | no (chop) |
| deck[4]  | P1 | **{R1, B1, P1}**       | yes (rank-1 + neg-Y) |
| deck[16] | B5 | any-5                  | yes |
| deck[17] | P2 | wide                   | no |

Stacks: R=1, Y=2, G=1, B=0, P=0. So deck[4] is "any 1 of {R,B,P}" — R1
is no longer playable, B1 and P1 are. The card is **possibly-playable** but
not strictly known-playable, so `PlayKnownPlayable` correctly does not fire.

The engine recommends `Discard deck[3]` (chop) instead of either:
- giving the obvious red play clue to Bob (touching his R2), or
- waiting for someone to disambiguate deck[4].

Test: `tests/replay_regression.rs::should_not_discard_for_no_good_reason`.

## What the search sees

```
total=52.175  Discard deck[3]                    (chosen)
   [0] DiscardChop      Discard chop (P3)
   [1] SimplePrompt     Clue purple → Alice {17,4}    ← Bob simulated to give this
   [2] DirectPlayClue   Clue red → Bob {9,7}
   [3] +1 PlayKnownPlayable  Alice plays deck[4] (now narrowed to P1)
   [4] +1 PlayKnownPlayable  Bob plays deck[7] (R2)
   [5] DiscardChop

total=46.314  Clue red to Bob                    (the move we wanted)
   [0] DirectPlayClue   Clue red → Bob {9,7}
   [1] +1 PlayKnownPlayable  Bob plays deck[7] (R2)
   [2] DiscardChop      Cathy discards deck[10] = R4    ← burns a critical
   [3] DiscardChop      Alice discards deck[3] = P3
   [4] DiscardChop      Bob   discards deck[8] = G3
   [5] DiscardChop      Cathy discards deck[11] = Y2
```

## What's happening mechanically

The projected play in the discard line is *mechanically valid*: each
simulated turn uses the acting player's own POV. Bob really can see Alice's
deck[4]=P1 and deck[17]=P2, so from his POV "purple to Alice" is a sensible
prompt-style play clue. Alice's resulting empathy on deck[4] genuinely
narrows to `{P1}` once that clue arrives. No POV leak.

The bug is one level up: the search **trusts Bob will give exactly that
clue**, and uses that trust to discount Alice's own move at the root.

## Two specific failure modes

### 1. Asymmetric rollout policy

In the discard branch, the rollout includes Bob → `SimplePrompt`-purple →
Alice plays. In the play-clue branch, Bob plays R2 instead (because
`PlayKnownPlayable` outranks clueing for him), then Cathy faces the *same*
purple-clue opportunity Bob had in line 1 — and the rollout picks
`DiscardChop` for her instead. The result: line 1 gets 2 simulated plays,
line 2 gets 1, and the line-2 rollout cascades into 4 chop discards
including a critical R4.

The same purple-to-Alice clue is available in both branches; the rollout
just happens to pick it in one and not the other based on per-turn policy
ordering. That asymmetry, not the rollout score function, is the load-bearing
bug.

### 2. Real-H-group violation

Even if the rollout were symmetric, "I'll discard because my partner has a
clue available that would solve my hand" is anti-H-group. A human player
would clue Bob and let deck[4] resolve naturally a turn later. The
evaluator has no term for "don't burn a chop to outsource your own
information problem to your partner."

## Relationship to draw-dilution

Action totals:

| Line              | Plays | Discards | Clues | Total draws |
|-------------------|-------|----------|-------|-------------|
| Discard (chosen)  | 2     | 2        | 2     | **4**       |
| Play-clue         | 1     | 4        | 1     | **5**       |

The play-clue line draws one more card, so per `draw_dilution_bias.md` it
takes a ~0.3-point leaf hit from `team_empathy` / `critical_in_hand` /
`efficiency`. That accounts for ~5% of the 5.86-point gap. Dilution is
contributing but is not the dominant cause.

## Possible directions

Ranked roughly by invasiveness.

### A. Penalty for outsourced disambiguation (cheapest)

Extend `signal_ignored_penalty` (or add a sibling term) to charge the actor
when they discard while holding a touched card whose empathy would collapse
to a known playable under a single still-available clue. Makes the cost of
"let Bob solve this" explicit at the root, without restructuring the
rollout.

- Pros: localized, matches the H-group intuition, doesn't require
  rebalancing leaf weights.
- Cons: heuristic; needs care so it doesn't fire on cards that genuinely
  *should* be left for a teammate (e.g. a finesse target).

### B. Aggressive Good-Touch reinterpretation on play

When a card successfully plays, reapply GTP to all clue-touched non-known-
trash cards: the played identity now joins "things a touched card cannot be
a duplicate of." Under this rule, after R1 was played on turn 8, Alice's
deck[4] empathy would narrow from `{R1, B1, P1}` to `{B1, P1}` — both
playable, so `PlayKnownPlayable` fires at the root and the whole search
question disappears.

- Pros: addresses the specific shape of this bug structurally; matches how
  human H-group players read their own hand.
- Cons: changes empathy semantics; need to check that existing passing
  tests don't depend on the looser rule, and that the narrowing is sound
  (i.e. won't ever rule out the true identity).

This is the direction we're taking now. See implementation in this session.

### C. Symmetrize the rollout policy

Make the rollout's per-turn action selection match between siblings — if
`SimplePrompt`-purple is the top tech for Bob in the discard branch, the
same tech ranked the same way should appear in Cathy's enumeration in the
play-clue branch. Investigate why `DiscardChop` outranks it for Cathy.

- Pros: removes a class of "the search trusts my partner more in some
  branches than others" bugs, not just this one.
- Cons: bigger change; requires understanding the per-tech priority
  interplay during rollout.

### D. Charge for predicted teammate clues

Inside the rollout, weight teammate clues more skeptically than own
actions — e.g. apply a multiplier to immediate bonuses from non-acting
players, reflecting that we can't actually force them to play that clue.

- Pros: principled hedge against rollout overconfidence.
- Cons: hard to tune; risks defeating the search's ability to reason about
  cooperative sequences (which is the whole point of a multi-ply rollout).

## Recommendation

(B) is the right structural fix for the specific empathy gap and we're
doing it now. (A) is a worthwhile follow-up regardless — even with sharper
empathy there will be hands where one extra clue would unlock a play, and
"let the partner do it" should not be a free option at the root.

(C) is the deeper search-policy issue; worth investigating separately once
(A)/(B) land and we have a cleaner baseline.
