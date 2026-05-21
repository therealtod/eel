# `card_known_playable` deck-truth fallback leak

## Summary

After the cheating-leak refactor (see `cheating_leak_refactor_plan.md`),
`PlayerKnowledge` and `LightweightPlayerPOV` correctly separate direct
sight from clue/convention inference, and search rollouts intersect
visible-cards across observer and teammate.

However, `DefaultEvaluator::card_known_playable` (`src/engine/evaluator.rs:531`)
still has a deck-truth fallback that is **not gated on visibility**:

```rust
let bits = table_state.deck.get_global_empathy(deck_idx).as_bits();
bits != 0 && (bits & playable_mask) == bits
```

`get_global_empathy` collapses to the true card id once the deck slot
is resolved (i.e. once the card has been physically dealt). The
function does not check whether `pk` can actually see the card, so the
evaluator can decide "the holder knows this card is playable" using
information that's only available in the omniscient deck — classic
cheating leak, just on the evaluator side rather than the POV side.

The same fallback shape appears in:

- `card_known_playable` itself (used by `known_playable_in_hands` and
  `actor_has_known_playable`)
- inline at `evaluator.rs:499` (the loop that calls
  `card_known_playable` for the playable count)

## Why it wasn't fixed in the cheating-leak refactor

Adding a `visible_cards` gate (`(visible_cards >> deck_idx) & 1 != 0`
before the fallback) was tried during the refactor and **reverted**.
It broke pre-existing `critical_exposure_*` scenario tests that rely
on the deck-truth fallback firing for own-hand cards in scenarios with
no clue history. Those scenarios construct a table-state-only fixture
where the only "knowledge" the player has is the truth in the deck.

So the path forward is not a one-line gate — it requires deciding
what `card_known_playable` should return for cards the holder has no
inference or signal for. Options:

1. Gate on `visible_cards` and rewrite the affected scenarios to
   provide proper clues/inferences. Highest correctness, most work.
2. Gate on `visible_cards | own_hand` — special-case "own hand" so the
   scenarios keep working but teammates can't peek. Looks pragmatic
   but is probably wrong: own-hand cards are precisely the ones a
   player *cannot* see.
3. Replace the fallback with a check against the holder's effective
   inferred mask (i.e. `combined_possible_identities` minus the
   deck-truth tail) and accept that scenarios with no clue history
   will return `false`. Probably the right shape; needs scenario
   triage.

## Impact

Unknown but plausibly real in search: any leaf evaluation that
reaches `card_known_playable` for a teammate's hand could be reading
deck truth and so over-counting "known playable" cards. The earlier
POV-level fix prevents POV consumers from seeing through teammates,
but `card_known_playable` reads `PlayerKnowledge` directly and
bypasses the POV layer entirely.

## Suggested next step

Audit which evaluator callers actually reach the deck-truth branch
during a real search rollout (instrument and run a few scenarios),
then decide which of the three options above is least disruptive. If
the branch rarely fires in practice, option 1 + scenario rewrites is
likely fine.
