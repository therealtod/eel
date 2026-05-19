# Critical-exposure follow-ups: distribution (#3) and draw potential (#4)

Context: `DefaultEvaluator::critical_exposure_score` (in `evaluator.rs`) currently
implements points #1 (position) and #2 (buffer) of the user's design. Points #3 and
#4 are open. This document captures intended designs.

A live consequence of shipping #1+#2 only: the `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color`
test regresses, because the rank-1 line's leaf state has Cathy with no in-hand
buffer remaining (she played her 1s during the 6-ply rollout), so her chop is
read as "fully exposed." The color line keeps two known-playables on Cathy's hand
at the leaf → low exposure. The fix is downstream of this document: #3 and/or #4
should restore the right ranking.

---

## #3 — Penalise bad critical distribution

### Intent

A team-state where one player holds many critical cards is worse than one where
the same cards are spread across hands. Concentration increases the chance a
single discard turn loses multiple critical cards before clue tokens can save
them, and concentration reduces the cluer's options (the same player must give
all the saves).

### Proposed shape

Add a multiplicative concentration factor to `critical_exposure_score`:

```
per_player_threat[p] = Σ (threat of each critical card in p's hand)
total = Σ per_player_threat[p] * concentration_factor(per_player_threat[p])
```

`concentration_factor(x)` is `>= 1` and grows with `x`. Two candidates:

- **Quadratic**: `1 + alpha * (x - 1)` for `x > 1`, else `1`. Each extra critical
  in the same hand adds `alpha` to the multiplier. Simple to reason about.
- **Soft-exp**: `1 + alpha * (1 - exp(-x))`. Saturates — caps the concentration
  bonus so an already-stuck hand isn't double-punished beyond a point.

Start with quadratic, `alpha = 0.5`. Reassess against the regression test.

### Edge cases

- A hand with one critical contributes `threat * 1 = threat`. Unchanged.
- A hand with three criticals at threats `0.2 + 0.5 + 1.0 = 1.7` is multiplied by
  `1 + 0.5 * 0.7 = 1.35`. Total = `1.7 * 1.35 = 2.30`.
- Three players each holding one critical (each threat 0.57, sum 1.7): each
  multiplier = 1. Total = 1.7. Distributed is preferred by 0.6 points.

### Implementation notes

- Add a `concentration_factor` helper; keep the per-card threat formula intact.
- Compute `per_player_threat` inside the existing per-player loop before adding
  to `total`. Multiply by `concentration_factor(per_player_threat)` at the loop
  bottom.

### Tests to add

- Same total threat distributed across N players → lower than same total in one
  hand.
- One critical per player → factor exactly 1; no change from baseline.

---

## #4 — Reward draw potential when far from endgame

### Intent

Drawing a fresh card is not purely negative. Away from the endgame, the deck
contains many still-needed cards that could land in a player's hand and be
playable later. The current evaluator punishes draws (via various dilution
shapes including the now-fixed `team_empathy` and the new
`critical_exposure_score`'s "no buffer at leaf"). It does not reward the
inverse — the chance a draw produces a useful card.

### Proposed shape

A separate leaf-level bonus `draw_potential_score` that estimates the expected
value of *future* draws given the current deck composition and game phase:

```
draw_potential = deck_remaining
               * fraction_of_deck_that_is_useful(table_state, static_data)
               * gradient(distance_to_endgame)
```

Where:

- `deck_remaining = table_state.deck.current_size` — cards left to draw.
- `fraction_useful = count(still_needed_ids_in_deck) / deck_remaining`. A still-
  needed card is one whose copies aren't all played + discarded. (Computed from
  `card_copies_count_by_id` minus played minus discarded.)
- `gradient(d)`: a curve that is `~1.0` when many cards remain and decays to `0`
  near the endgame. Suggested: `gradient = min(1, deck_remaining / N)` where
  `N = num_players`. (Roughly: "draws matter until each player has played their
  last hand-refill.")

### Why this counteracts the #1+#2 regression

In the rank-1 test, the rank-1 line consumed three of Cathy's hand slots into
fresh draws. Each fresh draw has positive expected value when the deck is full
of still-needed cards. The current evaluator only sees the leaf state's
exposure — it doesn't price in the expectation. Adding this bonus rewards lines
that *exchange known cards for potentially-useful unknown ones*, which is
exactly what plays do.

### Implementation notes

- Cheap to compute: one pass over `card_copies_count_by_id` and the playing
  stacks. No per-player work.
- The bonus is a **state property** — symmetric across lines that reach the same
  table state. Don't structure it as a per-action delta; the leaf is enough.
- New weight `draw_potential_weight`. Start small (e.g., `0.1`) since the term
  scales with `deck_remaining` (~30 cards early game) and could dominate.

### Tests to add

- Deck full + 30 still-needed cards remaining → bonus positive and significant.
- Deck near empty → bonus ~0 regardless of useful-fraction.
- Deck full but all-still-needed are unreachable (all dead suits) → bonus 0.

---

## Ordering and rollout

1. Implement #4 first. It is a state-symmetric leaf term with no concentration
   logic; easier to test in isolation. Likely fixes the rank-1 regression on
   its own because the regressing line's leaf has higher `deck_remaining`
   (fewer draws used) and the leaf state has many still-needed cards. Wait —
   actually that's wrong. Rank-1 *consumes* draws, so its `deck_remaining` is
   lower than the color line's. Need to check the direction empirically before
   committing to this ordering.
2. Then #3. Concentration is a refinement on top of an already-working
   exposure term.
3. Tune `critical_exposure_weight`, `draw_potential_weight`, and
   `concentration alpha` jointly against the existing test suite.

### Open question

The rank-1 regression suggests the new exposure term *over-weights* lines whose
plays drained their hand buffer. Before #3/#4, it's worth checking whether
including **clue tokens as a soft buffer** in the existing formula would also
address the issue:

```
effective_buffer = hand_buffer + (clue_tokens_remaining / num_players)
```

Rationale: a clue token is a save-action ready to deploy. With many tokens, the
team can save any critical card before discard. This is conceptually adjacent
to #3 (distribution) and #4 (draw potential) but local and cheap.

If this single change closes the rank-1 regression, ship it as part of the
current iteration rather than waiting for #3/#4.
