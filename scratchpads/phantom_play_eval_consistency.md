# Phantom plays leak inconsistency into the leaf evaluator

## Summary

`KnowledgeAwareGameState::score(variant, phantom_plays)` counts phantom plays
as if they advanced stacks (`real_stacks + phantom_plays`). But `playable_mask`
— used by `known_playable_in_hands`, `critical_exposure_score`,
`misinformation_score`, and several upper-bound calculations — reads
`playing_stacks` directly, which only reflects **real** stack advancement.

The leaf evaluator therefore looks at two views of the same state:

| Term | Phantom-aware? | Source |
|---|---|---|
| `game_score` | ✅ yes | `state.score(variant, phantom_plays)` |
| `pace` | ✅ yes | `state.pace(phantom_plays)` |
| `efficiency_penalty` | ✅ yes | `state.required_efficiency(phantom_plays)` |
| `playable_mask` (used downstream) | ❌ no | `table_state.playable_cards()` |
| `known_playable_in_hands` | ❌ no | uses `playable_mask` |
| `critical_exposure_score` | ❌ no | uses `playable_mask` |
| `misinformation_score` | ❌ no | uses `playable_mask` |
| `upper_bound` heuristics | ❌ no | uses `playing_stacks` |

This shows up as a real bias in search ranking. Concrete example from
`should_play_known_playable` (turn 7):

- PLAY-line leaf: `game_score = 50` (3 effective plays = 1 real + 2 phantom),
  `playable_mask = {R2, Y1, G2, B2, P1}` (only real stacks).
- Cathy's `deck[14]` empathy = `{R2, Y2, G2, B2, P2}` (rank-2 clue, no Good
  Touch narrowing because phantom plays don't trigger reapply).
- Subset check `{R2,Y2,G2,B2,P2} ⊆ {R2,Y1,G2,B2,P1}` fails (Y2, P2 missing).
- `known_playable = 0` even though the team's reasoning says "this is a rank-2
  in a state where, by `game_score`, ~5 stacks have advanced."

The score breakdown then says "we played 5 cards" via `game_score=50` AND
simultaneously "no one is holding a known-playable" via `known_playable=0`.
These are inconsistent. In any actual consistent extension of the rollout,
**either** Cathy's `deck[14]` is in a suit whose phantom did advance (so it's
playable AND counted by `game_score`) **or** the suit didn't advance (so
`game_score` should not have been credited). Both terms claim authority over
the same set of phantom plays and disagree.

## Why this matters

It biases the search toward whichever line produces more phantom plays. A
line with `k` phantom plays gets:

- `game_score` credit: `+k × score_weight` (rewarded).
- `known_playable_in_hands` credit on the **same cards**: 0 (not rewarded).

Lines that play fewer phantoms but more reals look "worse" on `game_score` for
the same number of effective plays even though by reality they're equivalent.
And lines that play many phantoms see their hands depleted in `game_score`
without paying the inverse cost: nobody on the team is recognized as holding
known-playables for the suits whose stacks haven't actually advanced.

## How phantom plays currently work

From `KnowledgeAwareGameState::apply_play`
(`src/engine/knowledge_aware_game_state.rs` ~365–435):

1. Resolve the played card's effective identity:
   - **Singleton empathy** (or truth that contradicts the player's empathy):
     commit, real play, advance stack, reapply Good Touch.
   - **Ambiguous-but-all-playable empathy + play signal**: phantom play. Card
     removed from hand, stack does NOT advance, no Good Touch reapply,
     `is_phantom = true`.
   - **Truly unknown**: hidden-info path. Card removed, no score change.

2. The recursion in `tree_action_selection_strategy::best_score_at_depth`
   accumulates `phantom_plays` along the search path. The leaf evaluator
   receives this single accumulated `u8`.

3. The leaf evaluator threads `phantom_plays` into `state.score(...)`,
   `state.pace(...)`, `state.required_efficiency(...)` — but **not** into the
   playable-mask computations. They go through `table_state.playable_cards()`,
   which has no phantom awareness.

The deliberate design choice for phantom plays is "don't guess which stack
advanced, because committing would distort downstream playability/criticality
reasoning." The intent is sound: a guess that "Alice's deck[2] was Y1"
followed by reasoning "so now Y2 is playable for everyone" propagates a
fabrication. But the actual implementation is **half-applied**: it correctly
abstains from committing the stack assignment, but then loses sight of the
real information the phantom does carry, which is "one of these suit-rank
candidates was just consumed."

## Why the simple fix is wrong

The naive "widen `playable_mask` by `phantom_plays` ranks per suit" — i.e.
allow ranks `stack[s]+1+i` for `i ∈ 1..=phantom_plays` on every suit
independently — gives every card credit as if each phantom play could have
advanced every suit. That overcounts:

> 3 phantom plays into suits ⊆ {Y, G, P} **cannot** simultaneously make all
> three of {Y2, G2, P2} playable — at most each phantom advances one suit.

Tried this on `should_play_known_playable`:

| Leaf | `known_playable` before | `known_playable` after |
|---|---|---|
| DISCARD line (3 phantoms) | 0.0 | 5.0 |
| PLAY line (2 phantoms) | 0.0 | 2.0 |

The DISCARD line, with more phantoms, gained more credit — widening the gap
between the two PVs from 9.7 to 12.7 points in the wrong direction.

The fix expressed the right intuition ("phantom plays advance the team's
effective knowledge of stack progression") but the wrong arithmetic
(treating each phantom as independently advancing every suit).

## What a correct fix would look like

Phantom plays carry partial information: **one of the played card's empathy
candidates was consumed.** A consistent fix has two ingredients.

### 1. Track per-card phantom-empathy, not just a counter

Replace `phantom_plays: u8` with `phantom_assignments: SmallVec<[u64; N]>` —
a vector of empathy bitmasks, one per phantom play. Each entry says "this
phantom advanced one of these suit-rank pairs." Equivalently a constraint:
exactly one bit in each mask was "consumed" as a stack advancement, and the
set of consumed bits is disjoint per-suit-rank.

### 2. Make playable-mask consumers consistent with that constraint

Three semantics options for `known_playable_in_hands`, in order of
sophistication:

**(a) Existential (loose):** card counts as known-playable if there exists a
consistent assignment of the phantoms under which the card's empathy ⊆
playable. Cheap upper bound; closely tracks "the team's optimistic reading."

**(b) Universal (tight):** card counts as known-playable iff for every
consistent phantom assignment, the card's empathy ⊆ playable. Conservative
lower bound; matches "the team is certain it's playable regardless of which
suit was advanced."

**(c) Probabilistic:** weight by `P(card is playable | phantom assignment)`
across the assignment distribution. Most defensible but more expensive and
needs a prior over assignments.

Same choice extends to `critical_exposure_score`, `misinformation_score`,
etc. — any term that takes a position on "is this card playable / critical /
trash" should ask the same question consistently.

A reasonable starting point: **existential for known-playable, universal for
critical/misinformation.** Reward the team for cards that *could* be useful
under any consistent extension; charge them for cards that are *necessarily*
bad under every consistent extension.

### 3. Don't break `game_score`

`state.score(variant, phantom_plays)` is currently `real_stacks +
phantom_plays`. That's existential too — credits the team with `phantom_plays`
worth of advancement assuming each phantom hit *some* stack. Compatible with
(a). With (b) we'd need to back off `game_score` similarly. Cleanest is to
keep `game_score` existential and use option (a) for the in-hand terms so the
two views agree.

## Open questions

- **How does Good-Touch reapply fit in?** Currently gated on
  `stack_advanced`, which is false for phantom plays. Should the team's
  empathy narrow on a phantom play? "One of the played card's empathy
  candidates is now trash" is true — though we don't know which. The
  narrowing would be conditional / disjunctive. Adjacent to but not
  identical to the playable-mask question.

- **Per-suit phantom counts as a cheaper proxy?** Instead of full per-card
  empathy tracking, just store `phantom_advances_per_suit: [u8; NUM_SUITS]`.
  Conservative: increment all suits in the played card's empathy. Gives a
  per-suit cap on "how much could this suit have advanced." Sufficient for
  many cases without the full constraint-satisfaction.

- **Upper-bound heuristics.** `evaluator.upper_bound` is used for branch-
  and-bound pruning. It probably wants the optimistic (existential) view,
  but currently reads only real stacks. Inconsistency here could mean we're
  pruning candidates that an honest upper-bound would keep.

- **Interaction with `clue_demand` / `efficiency_penalty`.** These currently
  use phantom-aware `pace` and `required_efficiency`. Consistent with the
  existential reading. If we shift any term to universal, those should
  match.

## Relationship to other scratchpads

- `known_playable_silent_zero.md`: the surfacing symptom that led here.
  Documents the regression test and the failed naive fix. This doc is the
  more general framing.

- `draw_dilution_bias.md`: a different leaf-bias problem. The "two lines
  end at the same `game_score`" framing assumes `game_score` is well-defined
  for both lines — which it is, modulo the phantom-counter accounting
  described here.

- `rollout_outsources_disambiguation.md`: also a search-fidelity issue but
  about per-turn policy choices, not leaf evaluation. Independent.

- `critical_exposure_followups.md` #4 (draw potential): is a state-symmetric
  leaf bonus. Its design also reads `playable_cards` / `still_needed` —
  would inherit any phantom-awareness fix here.

## Decision

**Not blocking on this for the current regression test.** Issue B (chop-
discard pricing) is the smaller-blast-radius lever for closing
`should_play_known_playable`. This scratchpad captures the structural gap
for a separate, larger investigation when we have the appetite for a search-
layer refactor.

If/when picked up: start by adding a per-suit phantom-advance counter as
the cheap proxy. Wire it through `playable_cards` (via a new method,
`phantom_aware_playable_cards`, leaving the original intact for callers
that explicitly want real-only). Re-run the regression suite; this is the
diagnostic. If results look right, then consider whether to graduate to
per-card empathy tracking.
