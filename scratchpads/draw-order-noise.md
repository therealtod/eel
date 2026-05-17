# Draw-order noise in leaf evaluation

## Problem

When two search lines differ in the timing of a discard, freshly-drawn search cards end up
with slightly different empathy sets at the leaf, because the deck-pool distribution shifts
depending on which cards were discarded *before* each draw.

Concretely: a card drawn from a pool that already has card X removed has marginally different
identity probabilities than a card drawn before X was removed. This is reflected in
`team_empathy_score` and `critical_in_hand`, which both depend on `empathy.count_possibilities()`.
Freshly-drawn cards contribute nothing to `misinformation_score` (omniscient deck has no
singleton for them), so the noise is confined to those two terms.

## Why it matters

Any two lines that are strategically equivalent but differ only in when a discard occurs will
have their leaf scores perturbed by this deck-composition noise rather than by anything
meaningful. The perturbation is small (~0.005 in the `avoid_killing_critical` case) but enough
to break ties in the wrong direction.

## Observed instance

In the `understands_that_the_efficient_clue_loses_max_score` regression test:
- Discard-now line: Donald draws deck_index 28 (Alice's discard already removed from pool)
- CriticalSave-now line: Donald draws deck_index 27 (Alice's discard still in pool)
The Discard line wins by 0.005 purely because of this effect.

## Possible fix directions

- Normalise freshly-drawn card contributions in `team_empathy_score` and `critical_in_hand`
  so that cards with fully-open empathy (i.e. not yet a singleton in the omniscient deck)
  contribute a fixed expected value based on deck composition rather than a per-card empathy
  count that varies with draw order.
- Alternatively, exclude freshly-drawn search cards from those two terms entirely and rely on
  the other leaf signals (score, ceiling, pace) to differentiate lines.
