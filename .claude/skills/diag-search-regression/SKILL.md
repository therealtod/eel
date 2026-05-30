---
name: diag-search-regression
description: |
  Diagnose a failing tests/search_regression.rs test by reading the candidate_scored
  DEBUG output — decompose each candidate's total score, isolate the immediate_bonus
  step driving the wrong choice, and map it to the responsible evaluator term or
  scenario defect. Use when the engine picks the wrong action in a scenario-driven
  test and you need to understand WHY the ranking went wrong.
---

# Diagnose a search regression failure

A `tests/search_regression.rs` failure tells you *what* the engine chose, not *why* it
ranked that action above the expected one. The answer lives in the `candidate_scored`
DEBUG lines the test already emits — no extra diagnostic code needed.

## Step 1 — Run the test and read the DEBUG output

```
cargo test <test_name> 2>&1
```

The test binary emits one `candidate_scored` line per root candidate, e.g.:

```
DEBUG scored_actions{player=0 candidates=2}: eel::search: candidate_scored
  action=Play { card_deck_index: 3, .. }  tech="PlayKnownPlayable"
  leaf_score=-5.829  immediate_bonus=1.0  total=-4.829
  leaf=total=13.18 [score=10.0 -strike=0.0 +pace=3.0 …]
  line=[
    LineStep { action: Play { .. },   tech_name: "PlayKnownPlayable", immediate_bonus: 1.0   },
    LineStep { action: Discard { .. }, tech_name: "DiscardChop",       immediate_bonus: -0.12 },
    LineStep { action: Clue { .. },   tech_name: "LowLevelStall",      immediate_bonus: -19.4 },
    …
  ]
```

> [!TIP]
> **Debugging Inner-Ply / Subjective Decisions:**
> If the wrong choice is made at an inner ply of the PV (i.e. by a simulated player Bob or Cathy), compile with the `search-diagnostic` Cargo feature:
> ```bash
> cargo test <test_name> --features search-diagnostic 2>&1
> ```
> This prints the candidate actions proposed (`subjective_candidates_list`) and their full evaluations (`subjective_candidate_evaluated`) for all simulated players, making it extremely easy to see why a simulated teammate picked or rejected a specific action.

> [!NOTE]
> **Deck indexes are not the JSON listed order.** Before reasoning about which card a
> `touched_card_deck_indexes` set refers to, decode the mapping. The scenario loader
> (`src/engine/replay/from_scenario.rs`) assigns `deck_idx = player_range[hand_size − 1 − slot_pos]`,
> so the **first card listed in a JSON hand is the *highest* deck index** in that player's
> range. Example: Cathy's hand `["y1","r1","p3","b5","g1"]` with range `10..=14` maps to
> `y1=14, r1=13, p3=12, b5=11, g1=10` — so a rank-1 clue touching `[14,13,10]` is the three
> 1s. Cross-check against the clue-touch sets in the PV to confirm.


## Step 2 — Decompose the scores

The arithmetic:

```
total          = leaf_score + root_immediate_bonus
leaf_score     = leaf_breakdown.total + Σ(inner_immediate_bonuses)
                                            ↑ all line steps except the root
```

So `leaf_breakdown.total` is the objective leaf evaluation, and all the `immediate_bonus`
values along the line add (or subtract) on top of it. If two candidates have similar
`leaf_breakdown.total` but very different `total`, the discrepancy lives in the bonuses,
not the leaf evaluation.

Compute `Σ(inner_immediates)` by hand for each candidate — the outlier step is usually
obvious (one value is 10–20× larger than the rest).

> [!IMPORTANT]
> **Not every culprit is an immediate-bonus outlier.** When the totals are *close* and no
> single line step stands out (all immediates are normal ~±2), the deciding factor is
> almost always a **leaf term**. Diff the two candidates' `leaf=total=… [score=… +play=… …]`
> breakdowns **column-by-column** — the gap will localize to one leaf term (commonly
> `+play` = `known_playable`, `-crit_exp`, or `-eff`). Trust the printed breakdown numbers
> over hand-counting the leaf state; they are authoritative. See Step 3 for the
> label→field map and the horizon-saturation failure shape below.

## Step 3 — Identify the culprit bonus

`immediate_bonus` for each step is computed by `TreeActionSelectionStrategy::immediate_action_bonus`
(in `src/engine/tree_action_selection_strategy.rs`). It sums:

| Component | Non-zero when | Responsible function |
|---|---|---|
| `clue_bonus` | action is a Clue | `evaluator.clue_precision_bonus` |
| `signal_penalty` | actor has Signal::Play but takes non-play action | `evaluator.signal_ignored_penalty` |
| `play_bonus` | action is a Play | `evaluator.play_progress_bonus` |
| `team_empathy_bonus` | clue changes empathy for others | `evaluator.team_empathy_delta_bonus` |
| `discard_penalty` | action is a Discard | `evaluator.discard_action_penalty` |
| `critical_exposure_delta` | play/discard changes critical exposure | `evaluator.critical_exposure_delta_bonus` |

The most common large-magnitude culprits:

- **`clue_bonus` ≈ −19 to −20**: A clue touched a card whose exact identity is already
  present in the same hand (duplicate) or already fully played. `count_bad_touches` found
  1 bad touch; penalty = `good_touch_penalty × 1` = 20.0 by default. Check the touched
  deck indexes against the hand — are two cards the same identity?

- **`discard_penalty` ≈ −8**: `discard_action_penalty` fired. Either `bottom_deck_risk_score`
  is high (the discarded card's global empathy overlaps a semi-critical identity) or
  `discard_while_known_playable_penalty` fired (actor has a known playable in the search
  state). Check which sub-term applies.

- **`signal_penalty` ≈ −N`: `signal_ignored_penalty` fired. The actor has a `Signal::Play`
  tagged on an own-hand card but chose Discard or a wrong Play. This is usually correct
  behavior — it penalises path branches where an inner player ignores their own signal.

### Leaf-breakdown terms (the `leaf=total=… […]` string)

If Step 2 pointed you at a **leaf** term rather than an immediate step, map the short label
in the `leaf=…` breakdown to its field and the function that computes it (defined in
`src/engine/evaluator/types.rs` `Display` and `default_evaluator.rs`):

| Label | Field | Computed by | Notes |
|---|---|---|---|
| `score` | `game_score` | played-stack score × `score_weight` | **Horizon-capped** — both candidates often tie here |
| `+play` | `known_playable` | `known_playable_in_hands` | Leaf reward for cards still *in hand* known playable |
| `-crit_exp` | `critical_exposure_penalty` | `critical_exposure_score` | |
| `-eff` | `efficiency_penalty` | `required_efficiency` | |
| `+clue` | `clue_tokens` | `harmonic(count) × demand` | |
| `±crit_delta` | `critical_exposure_delta` | `critical_exposure_delta_bonus` | Immediate Δ folded in; 0.0 at a pure leaf |
| `-tempo` | `tempo_cost` | `tempo_cost` | Immediate; 0.0 at a pure leaf |

> [!WARNING]
> **`+play` (leaf) ≠ `play_bonus` (line step).** The leaf `+play` term is `known_playable`
> (`known_playable_in_hands`) — a reward for playable cards *held in hand* at the leaf. The
> per-step `play_bonus` is `play_progress_bonus` — the reward for the *act* of playing. Two
> different mechanisms at two different stages; don't conflate them. A line that *defers*
> plays scores **lower** `play_bonus` timing but **higher** leaf `+play` (more cards left
> banked unplayed) — see the horizon-saturation row in Common failure shapes.

## Step 4 — Understand the inner-player POV restriction

When the search simulates inner players (not the root actor), it builds their POV with:

```
effective_visible = root_visible ∩ active_pk.visible_cards
```

This prevents the "cheating bot" leak: the root player cannot know their own cards, so
inner players are also forbidden from seeing the root player's hand. Concretely:

- Inner Bob can see Cathy's hand (both root Alice and Bob can see it).
- Inner Bob **cannot** see Alice's hand (Alice is the root and cannot see herself).
- Newly drawn cards (deck indexes beyond the initial deal) are not in `root_visible` —
  inner players cannot see those either.

This restriction controls which techs fire for inner players and which stall paths they
take. If an inner player gives a bad LowLevelStall clue, it's usually because their
`effective_visible` excludes the cards that would have satisfied priority 1 (rank-5).

## Step 5 — Decide: scenario bug vs. evaluator bug vs. correct behavior

**Scenario bug** (most common for new tests):
- The hand layout creates an unintended asymmetry between the two paths — e.g. a
  duplicate card in one player's hand makes the stall clue bad only in the play-first
  path, not the discard-first path.
- Fix: redesign the hand to eliminate the asymmetry. Avoid duplicate card identities
  within any single hand. Check that the stall clues in each candidate's PV are clean
  (no bad touches, no -19 penalty).

**Evaluator bug**:
- A penalty is applied to a case it shouldn't cover, or a weight is miscalibrated.
- Confirm by checking whether the penalty formula in `evaluator.rs` actually describes
  the situation correctly, not just numerically matches.

**Correct behavior** (test expectation wrong):
- The engine is right and the test is testing a wrong assumption. Re-examine whether
  the scenario actually forces the expected action unambiguously.

## Step 6 — Verify the fix

After changing the scenario JSON or evaluator, re-run:

```
cargo test <test_name> 2>&1
```

Also run the full search regression suite to confirm no regressions:

```
cargo test --test search_regression 2>&1
```

## Common failure shapes

| PV symptom | Likely cause |
|---|---|
| One candidate has a step with `immediate_bonus ≈ −19` on a LowLevelStall clue | Stall clue touches two cards with same identity in the same hand (bad touch). Fix the scenario hand. |
| Discard candidate wins with large negative `discard_penalty` still beating play | Play path has a worse subtree (check for stall asymmetry). Or play's `leaf_breakdown.total` is much lower. |
| Two candidates have nearly equal `total` (difference < 0.5), and their `leaf=…` breakdowns differ in exactly one leaf term | **Real, deterministic leaf-term delta — not enumeration order.** Diff the breakdowns column-by-column (Step 2/3) and fix the responsible leaf term. Only suspect enumeration order if the breakdowns are *identical*. |
| Both candidates reach the **same capped `game_score`**; winner decided by a small `+play` (`known_playable`) delta | **Horizon-saturation artifact.** Both lines play the same cards within the horizon, so `game_score` ties; the line that *defers* its plays banks extra un-cashed playables in hand and wins on leaf `+play`. The efficient line is penalised for cashing progress. Levers: `play_deferral_weight` (discount deferred plays — already in `immediate_action_bonus`), or discount banked `known_playable` at the leaf. |
| Inner player takes a suboptimal action in one path's PV but not the other | `effective_visible` asymmetry: the root player has different knowledge along each branch. Check what the inner player can actually see. |
| Inner player gives a stall when a save/play should be available | Save tech requires seeing a critical card on chop. If that card is in the root player's own hand, `effective_visible` hides it from the inner player. |

## Relationship to `diag-replay-state`

Use `diag-replay-state` when the wrong choice is driven by corrupted empathy, bad
hypothesis state, or wrong tech proposals — things that require the full per-card state
dump at a specific turn.

Use this skill when the wrong choice is a **ranking issue**: all techs propose sensible
actions but the search scores them in the wrong order. The `candidate_scored` lines
already contain everything you need; no additional diagnostic code is required.
