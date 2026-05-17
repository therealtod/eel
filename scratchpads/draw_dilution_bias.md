# Draw-dilution bias in the leaf evaluator

## Symptom

Replay `refuses_to_play_known_playable.json`, turn 5. Cathy has a known-playable
y2 at deck index 14 (touched the previous turn by a yellow clue from Bob;
empathy singleton = y2; y1 already on the stack). She should play it. The
engine instead recommends `Discard { card_deck_index: 10 }` (chop trash g3).

Test: `tests/replay_regression.rs::should_play_known_playable_instead_of_discarding`.

## Diagnostic output

State at the failing turn is correct — empathy on deck[14] is a singleton y2,
and `PlayKnownPlayable` correctly proposes the play. The issue lives in the
search ranking. Top two candidates from `scored_actions`:

```
total=39.949 Discard { card_deck_index: 10 }
  leaf: total=38.95 [score=30.0 +pace=3.0 -eff=1.6 +crit=4.4 +clue=1.7 +team_emp=1.4 ...]
  [0] +0.00 DiscardChop  Discard cathy chop
  [1] +0.00 DiscardChop  Discard alice chop
  [2] +0.00 FiveSave     Clue rank 5 to cathy
  [3] +1.00 PlayKnownPlayable  Play y2  (deck[14])
  [4] +0.00 DiscardChop  Discard alice
  [5] +0.00 TwoSave      Clue rank 2 to alice

total=39.478 Play { card_deck_index: 14 }
  leaf: total=38.48 [score=30.0 +pace=3.0 -eff=1.7 +crit=4.3 +clue=1.8 +team_emp=1.2 ...]
  [0] +1.00 PlayKnownPlayable  Play y2  (deck[14])
  [1] +0.00 DiscardChop
  [2] +0.00 DiscardChop
  [3] +0.00 DiscardChop
  [4] +0.00 DiscardChop
  [5] +0.00 TwoSave
```

Both lines reach `game_score = 30` (3 cards played) and both honor exactly one
`play_progress_bonus = +1.00`. The 0.47-point gap is entirely in the leaf
evaluation of state shape.

## Action-count diff

| Line              | Plays | Discards | Clues | **Total draws** |
|-------------------|-------|----------|-------|------------------|
| Discard-then-play | 1     | 3        | 2     | **4**            |
| Play-now          | 1     | 4        | 1     | **5**            |

Clues don't draw. Both lines end with the same Hanabi score; play-now has drawn
**one extra card** from the deck.

## Per-term attribution of the 0.47-point gap

| Term           | discard line | play-now line | Δ play-now |
|----------------|--------------|---------------|------------|
| `team_empathy` | 1.4          | 1.2           | **−0.2**   |
| `crit_in_hand` | 4.4          | 4.3           | **−0.1**   |
| `efficiency`   | −1.6         | −1.7          | **−0.1**   |
| `clue_tokens`  | 1.7          | 1.8           | +0.1       |

Sum ≈ −0.3, matching the observed gap modulo small float rounding across terms.

### Why each term shifts

1. **`team_empathy_score` (−0.2).** Sums `(max_ids − popcount) / max_ids` across
   every own-hand card. A freshly-drawn card has `popcount ≈ max_ids` →
   contributes ≈ 0. Replacing a clued / partially-narrowed card with a fresh
   unknown strictly lowers the sum.

2. **`critical_in_hand` (−0.1).** Sums `overlap_bits / total_possibilities` per
   card. A fresh draw has a huge denominator, so any critical overlap is
   vanishing. Cards being *replaced* in the other line still had narrower
   empathy and contributed more.

3. **`efficiency_penalty` (−0.1, i.e. heavier penalty).** `required_efficiency`
   uses remaining deck size + cards still needed. One more draw = one fewer
   deck card = tighter efficiency requirement.

4. **`clue_tokens` (+0.1).** Play-now did 4 discards (+4 tokens); discard-line
   did 3 discards + 1 save clue (+3, −1). Play-now ends with more tokens —
   the only direction the extra draw helps. Partially offsets the dilution
   losses but not enough.

Net: drawing burns a slot of known information and replaces it with maximum
uncertainty. Several terms read that uncertainty as worse state.

## Why `play_progress_bonus` doesn't save us

`play_progress_weight = 1.0` exists precisely as a counter for this bias. Its
docstring (`src/engine/evaluator.rs`):

> Counteracts the structural bias where lines that play more cards lose ~1 pace
> point per extra draw at the leaf.

But it only fires when *play count* differs between lines. Here both lines
play the same card, just at different turns, so the bonus is symmetric (+1.0
on both sides) and the leaf bias goes uncompensated. The bandaid plugs one
hole and leaves the adjacent one open.

## What the bias is actually signalling

There is genuine information in the dilution: a fresh draw really is more
uncertain than the card it replaced, and that uncertainty really does cost
you on a future turn. The evaluator isn't wrong to notice it.

The bug is that it counts the cost *without counting the corresponding
benefit*: the played card is gone from hand, freeing a slot, and the deck is
one card closer to running out (which the team wants — `pace` and
`efficiency` ultimately resolve at game end). The leaf terms see the new
ambiguity without seeing that the *alternative* line is just deferring the
same cost to a later turn.

## Possible directions

Ranked by invasiveness:

### 1. Normalize empathy terms per hand card

`team_empathy_score` currently sums; make it an *average* per hand card.
A fresh unknown pulls the average toward 0 but a smaller hand pulls it back
up, so dilution cancels out. Same idea for `critical_in_hand`.

- Pros: clean semantic fix — these terms conceptually measure "how well do we
  know our hands on average," not "raw sum that grows with hand size".
- Cons: changes the relative scale of these terms; may need re-tuning against
  other weights (`score_weight`, `lost_score_ceiling_weight`, etc.).

### 2. Exclude freshly-drawn cards from empathy terms

Filter out cards whose global empathy is still maximally wide (or whose
`deck_index >= next_deck_index_at_root`). Removes the dilution term cleanly
without rescaling existing weights.

- Pros: smallest behavioral footprint; targeted at the actual source.
- Cons: loses signal for cards that genuinely got *partially* clued during
  search; coarse threshold.

### 3. Evaluate leaves at a shared time horizon

Compare lines by "same number of cards drawn" rather than "same number of
plies". Either extend shorter lines with extra simulated plies until draw
counts match, or evaluate at a fixed draw-count horizon.

- Pros: most principled — removes the apples-vs-oranges comparison entirely.
- Cons: structurally bigger change; requires rethinking how the search expands
  candidates and reports leaves.

### 4. Lift the `pace` clamp

`pace` is clamped to `num_players` from above, so the play-now line's genuine
pace gain (1 fewer card left to draw) is invisible at the leaf. Unclamping
would let the play-now line earn back some of its loss.

- Pros: trivial change; addresses one symptom.
- Cons: doesn't address dilution in `team_empathy` / `crit`; only partial fix.

### Recommendation

(1) Normalize empathy terms per-card. The whole point of `team_empathy_score`
and `critical_in_hand` is "average information per held card," not "total
across an arbitrary-sized hand." Per-card normalization fixes the dilution
*structurally* rather than via a counter-bonus, and the rescaling against
other weights is a one-time tuning pass.

(4) is a worthwhile cheap companion change regardless: the pace clamp throws
away a real, asymmetric signal.

## Out of scope here

This document is about the evaluator structure. A separate (cheaper but more
ad-hoc) fix is to extend `signal_ignored_penalty` so it also charges the
actor for ignoring a touched known-playable in their own hand — that would
make the failing test pass via an urgency penalty rather than by fixing the
underlying bias. Worth keeping in mind as a fallback if the evaluator
restructuring is deferred.
