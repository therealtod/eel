# Plan: Fix the "delayed-play" bias in the leaf evaluator

## Background

The search currently prefers a direct play clue over a more efficient finesse in
`tests/scenarios/search/avoid_stealing_finesse/`. Both lines reach game score 7
at the leaf, but the finesse loses by ~1 point because of a structural side
effect of *progress*: the finesse line consumes 1 extra deck card across the
6-ply horizon (5 draws vs 4), which costs −1.0 on `pace` and −1.2 on
`critical_in_hand`, only partly recovered by +1.2 on `clue_tokens`.

The deeper pattern (independent of this scenario): the leaf evaluator gives
**different per-card contributions to cards still in hand depending on whether
those cards were already identified pre-search or freshly drawn during search**.
This is a horizon-effect-style bias that rewards "delay your plays so they
happen after the horizon."

Two fixes are planned:

- **Plan A (principled, primary):** marginalize per-card contributions in the
  evaluator so that a freshly-drawn card and an identified card with the same
  *expected* properties contribute the same amount. Reread of `critical_cards_in_hand`
  (evaluator.rs:237) shows the math is already a `overlap / possibilities`
  expectation, but other terms aren't, and the *population* against which the
  expectation is taken (the deck pool) drifts between sibling search branches
  in ways that survive the per-card averaging.
- **Plan B (additive, secondary):** add a small per-play **tempo bonus** along
  the search line so a line that pushes more cards onto the stacks within the
  horizon is rewarded for the progress itself, not just for the final score.

Both fixes are valuable in combination: A removes the spurious per-card swing,
B rewards genuine forward progress so the engine isn't indifferent between
"play now" and "play after the horizon" when expected leaf value is similar.

---

## Glossary

- **Per-card eval term:** any term in `DefaultEvaluator::score_breakdown`
  (src/engine/evaluator.rs:493) that loops over hand cards and accumulates a
  scalar based on each card's empathy / inferred identity. Today these are
  `critical_in_hand`, `empathy_bonus`, `known_playable`, `team_empathy`,
  `resolved_cards`, and `misinformation_penalty`.
- **Realized contribution:** what a per-card term returns when the card has
  empathy popcount = 1 (fully identified) — typically 0 or 1, with no
  averaging.
- **Marginalized contribution:** the *expected* value of the realized
  contribution over the population of identities the card could plausibly
  be, given the leaf's deck pool and the player's empathy/inferred knowledge.
- **Deck pool at the leaf:** the multiset of card-IDs still in the deck and in
  hands that haven't been revealed (to spectators) at the leaf state. During
  search this is *not* the same as the omniscient deck's `empathy_by_index`,
  because `update_with_play_action` and `update_with_discard_action` can be
  called without identity (`apply_play` falls back when the actor's own
  knowledge can't pin the played card).

---

# Plan A — Marginalize unknown draws in the evaluator

## Goal

Make every per-card eval term invariant to whether a card in hand is
"freshly drawn during search" vs "already-identified pre-search," when the two
are statistically equivalent (same empathy distribution against the same deck
pool).

Concretely: if you take a leaf state and replace one hand card whose true
identity is `R3` (with empathy popcount = 1) by a *fresh draw* whose empathy
distribution exactly contains `R3` and 19 other still-possible identities, the
*expected* per-card contribution must equal what `R3` contributed in the first
state. Today some terms differ.

## Principle

For each per-card term that today emits a value of the form
`indicator(card has property P)`, replace it with
`P(card has property P | empathy, deck pool)`.

For terms already in expectation form (e.g. `critical_cards_in_hand` does
`overlap_count / popcount`), audit them to confirm the **population** used for
the expectation is the deck pool, not just the empathy mask — these can
disagree when search has anonymous plays/discards that don't narrow `Deck::empathy_by_index`.

## Step 1 — Build a shared "expected identity distribution" helper

Add a function on `Deck` (or a free function in `evaluator.rs` that consumes
`Deck` + `TableState` + `Variant`) with signature roughly:

```rust
/// For a card at `deck_idx`, return the probability that it has each card-id
/// in the variant, given:
/// 1. Its empathy mask (`Deck::get_global_empathy`).
/// 2. The remaining deck pool — i.e. variant.card_copies_count_by_id minus
///    `revealed_copies_per_index`, minus any anonymous discards / plays the
///    search has performed (these subtract from "remaining" but not from
///    revealed_copies, so the deck mask alone undercounts).
///
/// Returns an array of f64 of length `variant.number_of_suits * variant.stacks_size`.
fn expected_identity_distribution(
    deck_idx: CardDeckIndex,
    table_state: &TableState,
    variant: &Variant,
) -> [f64; MAX_UNIQUE_CARDS_IN_DECK];
```

Implementation:
1. Read `empathy = deck.get_global_empathy(deck_idx)`. Iterate the bits set in
   `empathy`. Skip identities that have all copies already accounted for in
   `discard_pile.copies_of(id) + playing_stacks` (impossible draws).
2. For each remaining identity `i`, weight by the number of copies of `i` that
   are still in the unrevealed pool: `total_copies[i] - revealed_copies[i] -
   anonymous_consumed[i]`. The first two are easy; `anonymous_consumed[i]` is
   the tricky part — see step 1a.
3. Normalize to a probability vector. If the sum is 0 (degenerate state), fall
   back to uniform over `empathy` bits.

### Step 1a — Track anonymous consumption

Today, `TableState::update_with_discard_action` (table_state.rs:144) calls
`discard_pile.add_card()` without an id, and
`update_with_play_action` (table_state.rs:107) doesn't update the discard
pile at all. So in search, the *count* of anonymous discards is recoverable
from `discard_pile.total_size()` minus the per-id sum from
`copies_of(id)`, but the *which ids* part is lost.

Decision point:

- **Option 1 (cheap):** Subtract anonymous consumption *uniformly* across the
  empathy mask. This is the maximum-entropy assumption: each unidentified
  consumed card was equally likely any of the remaining identities. Easy to
  implement, slightly inaccurate.
- **Option 2 (accurate):** Make `apply_play` / `apply_discard` always reveal
  identities during search. Today they only reveal when the actor's knowledge
  can pin the identity (apply_play falls back to anonymous when it can't).
  In search we typically have an omniscient view that *could* pin every card
  — verify this assumption first. If true, force-reveal and the empathy mask
  always reflects truth.

**Recommendation:** start with Option 1 because it doesn't require touching
the action-application path. Open an exploratory task to verify whether
Option 2 is even meaningful (the search may genuinely operate without
omniscient ids).

## Step 2 — Audit each per-card term

For each of the six per-card terms, decide:
- Is it currently expectation-correct against the deck pool?
- If not, replace its body with a call to `expected_identity_distribution`.

| Term | Helper today | Status | Action |
|------|--------------|--------|--------|
| `critical_in_hand` | `critical_cards_in_hand` (evaluator.rs:237) | Uses `overlap/popcount` against `deck.get_global_empathy` — already a form of expectation, but population mismatch when anonymous consumption has occurred. | Replace `popcount` denominator with `expected_identity_distribution` so the population matches the actual pool. Critical-ness is a property of identities (a singleton mask `critical_mask`), so contribution = `Σ_i P(card = i) · 1[i in critical_mask]`. |
| `team_empathy` | `team_empathy_score` (evaluator.rs:386) | Uses popcount of `combined_possible_identities`. A fully-resolved card gives near-1; a fresh draw gives near-0. Asymmetric and the source of the swing. | Replace with `(max_identities − E[popcount]) / max_identities` where the expectation is over the player's empathy. For a fresh own-hand card with no convention info, this gives ~0 (correct: no info reduction). For a resolved card, ~1 (correct). The fix is mainly conceptual cleanup; the magnitude only swings when convention narrows the mask. |
| `resolved_cards` | `resolved_card_count` (evaluator.rs:463) | Hard indicator `popcount == 1`. Highly asymmetric: 1.0 if resolved, 0.0 otherwise. | Either (a) drop this term entirely since `team_empathy` covers the same thing more smoothly, or (b) replace with `P(empathy collapses to singleton)` which is 1.0 only when popcount=1 already → same as today. **Recommend (a)**: delete `resolved_cards_weight` and let `team_empathy` carry the signal. Less code, less double-counting. |
| `known_playable` | `known_playable_in_hands` (evaluator.rs:343) | Indicator: card is playable iff all empathy bits are in `playable_mask` (or has `Signal::Play`). A fresh draw is never "known playable." | Replace with `Σ_i P(card = i) · 1[i in playable_mask]`. For fresh draws this gives the natural "fraction of pool that is currently playable" — typically small but nonzero. For finesse-signal cards, the `Signal::Play` branch can keep its 1.0 contribution (it represents convention-level certainty, not population probability). |
| `empathy_bonus` | `empathy_precision` (evaluator.rs:311) | `max_identities − popcount` per clued card. Already disabled by default (`empathy_weight = 0.0`). | Audit but defer — disabled term, not contributing to the bug. Note in code that re-enabling requires marginalization first. |
| `misinformation_penalty` | `misinformation_score` (evaluator.rs:420) | Only fires when truth is a singleton in `deck.get_global_empathy` (revealed to spectators). Skips fresh draws explicitly. | Already correctly excluded for fresh draws by design (comment at evaluator.rs:419). No change needed, but add a regression test that exercises the boundary. |

## Step 3 — Refactor the loop

After the audit, the per-card loop body in each term collapses to a common
shape:

```rust
for hand in table_state.hands[..num_players].iter() {
    for &deck_idx in hand.cards() {
        let dist = expected_identity_distribution(deck_idx, table_state, variant);
        // term-specific reduction:
        // - critical_in_hand: Σ dist[i] * (1 if i in critical_mask else 0)
        // - known_playable:   Σ dist[i] * (1 if i in playable_mask else 0)
        // - team_empathy:     (max_identities − effective_popcount(dist)) / max_identities
        total += /* reduction */;
    }
}
```

Consolidating into a single loop that computes `dist` once and feeds multiple
reductions is a hot-path win (the distribution is recomputed today inside each
term). **Do this as a follow-up commit after correctness is established**, not
in the same change — it makes the diff harder to review.

## Step 4 — Tests

Add tests in `src/engine/evaluator.rs` under the existing `mod tests`:

1. **`marginalization_invariance_critical`** — Build two `TableState`s that
   differ only in whether one hand card has empathy popcount=1 (revealed `R3`)
   vs popcount=N (fresh draw whose mask contains `R3` and N-1 non-critical
   ids). With the deck pool set so `P(card = R3) = 1/N` in the fresh-draw
   state, assert that `critical_in_hand` contributes the same amount per
   card. (Today they don't — that's the bug.)
2. **`marginalization_invariance_known_playable`** — same shape but for
   `known_playable_in_hands`. A fresh-draw card should contribute
   `(playable_count / pool_count)` per slot.
3. **`team_empathy_no_credit_for_fresh_draws`** — assert a fully-unknown
   fresh-draw own-hand card contributes 0.0 (regression guard for
   `team_empathy_score` after the rewrite).
4. **`avoid_stealing_finesse` integration** — re-run
   `tests/search_regression.rs` for this scenario after the change. The
   currently-failing test should pass.

## Step 5 — Tuning pass

After the change, the magnitudes of `critical_in_hand_weight`,
`known_playable_weight`, and `team_empathy_weight` will shift because fresh
draws now contribute small positive amounts where they previously contributed
zero. Run the existing self-play binary (`src/bin/selfplay.rs`) on a sample of
seeds and compare to a baseline scored run.

Acceptance criterion: aggregate average score doesn't regress more than 0.2
points across 100 seeds. If it does, re-tune `critical_in_hand_weight` and
`known_playable_weight` downward proportionally (the per-card contribution is
now non-zero for more cards, so the total grows).

## Step 6 — Docs

Update:
- `docs/architecture/design.md` if it describes the evaluator's per-card
  treatment (verify; likely needs an "expected-value contribution" note).
- The doc comment on `DefaultEvaluator` (evaluator.rs:149-148) to describe
  the marginalization principle.

## Risks

- **Anonymous consumption tracking is approximate** with Option 1 in step 1a.
  Misestimates the deck pool when many anonymous plays/discards have happened.
  If this turns out to matter (verify via the self-play comparison in step 5),
  upgrade to Option 2.
- **Removing `resolved_cards_weight` is a behaviour change** even if the
  default is 0.0 today — check `Cargo.toml` and configs to see if any callers
  set it to non-zero. If so, leave the field with a doc note that says
  "subsumed by team_empathy; setting > 0 double-counts."
- **`known_playable` for fresh draws becomes non-zero**, which may make the
  search think "any fresh card might be playable" and over-value drawing.
  Watch for this in step 5.

## Estimated effort

- Step 1 (helper): 0.5 day
- Step 2 (audit & per-term changes): 1 day
- Step 3 (refactor): defer
- Step 4 (tests): 0.5 day
- Step 5 (tuning): 0.5 day, mostly compute time
- Step 6 (docs): 0.25 day

**Total: ~2.5 days** of focused work.

---

# Plan B — Add a tempo bonus along the search line

## Goal

Reward the search line for *cumulative progress made during the horizon*, not
just the leaf's `game_score`. Today, two lines that both reach game_score=7
look identical on the `game_score` term, and the choice between them collapses
to side effects (pace, crit, clues). A tempo bonus makes the line that played
more cards *during the search* score higher, directly offsetting the
"horizon-deferred play" preference.

## Where it goes

This is an **immediate (per-action) bonus**, applied along the search line —
analogous to the existing `clue_precision_bonus` (evaluator.rs:104, called
from `tree_action_selection_strategy.rs:299`). Per-action bonuses sum into
`ScoredNode::total_score` alongside the leaf `total`, so they're naturally
"line-cumulative" without any state plumbing.

## Step 1 — Add the trait method

In `src/engine/evaluator.rs`, add to the `Evaluator` trait:

```rust
/// Immediate bonus for a play action that successfully advances a stack.
/// Models the value of forward progress within the search horizon, separate
/// from the leaf `game_score` term (which is symmetric for lines that reach
/// the same total). Misplays (strikes) get 0 here; the strike penalty in the
/// leaf already handles them.
fn play_progress_bonus(
    &self,
    _action: &GameAction,
    _pre_action_state: &TableState,
    _post_action_state: &TableState,
    _static_data: &StaticGameData,
) -> Score {
    0.0
}
```

Default returns 0.0 so non-default evaluators are unaffected.

## Step 2 — Implement on `DefaultEvaluator`

Add a weight field:

```rust
/// Reward per card successfully played to the stacks within the search horizon.
/// Counteracts the structural bias where lines that play more cards lose ~1 pace
/// point per extra draw at the leaf. Tuned to make a play-now line beat a
/// play-after-horizon line when expected leaf scores are similar, without
/// overwhelming safety considerations.
///
/// Suggested default: 1.0. Set to 0 to disable.
pub play_progress_weight: f64,
```

Implementation:

```rust
fn play_progress_bonus(
    &self,
    action: &GameAction,
    pre: &TableState,
    post: &TableState,
    static_data: &StaticGameData,
) -> Score {
    if self.play_progress_weight == 0.0 {
        return 0.0;
    }
    let GameAction::Play { .. } = action else { return 0.0; };
    let advanced = post.score(&static_data.variant) > pre.score(&static_data.variant);
    if advanced {
        self.play_progress_weight
    } else {
        0.0
    }
}
```

Only successful plays count; misplays (strikes) are already penalised by the
leaf `strike_penalty` term — don't double-charge them.

## Step 3 — Wire into the search

In `src/engine/tree_action_selection_strategy.rs`, find
`immediate_action_bonus` (call site at line 299) and extend it to add
`play_progress_bonus` to the sum. The existing pattern is:

```rust
fn immediate_action_bonus(action, evaluator, before, after, static_data) -> Score {
    let clue_bonus = match action {
        GameAction::Clue { .. } => evaluator.clue_precision_bonus(...),
        _ => 0.0,
    };
    let signal_penalty = evaluator.signal_ignored_penalty(...);
    clue_bonus + signal_penalty
}
```

Add a `play_bonus` line and include it in the sum. The function needs the
pre- and post-action `TableState` — verify both are available at the call
site (they appear to be: `root_state.table_state` and `next.table_state`).

## Step 4 — Pick the magnitude

The user's analysis shows the structural per-extra-draw cost at the leaf is
~−1.0 (pace) plus ~−1.2 (crit), partially offset by ~+1.2 (clue). Net per
extra play during horizon: ~−1.0 of "structural drag."

Setting `play_progress_weight = 1.0` exactly counteracts that drag, leaving
the choice to be decided by the leaf's *substantive* signals (score,
ceiling, misinformation, signal-ignored). Recommend **starting at 1.0** and
verifying on the failing scenario.

## Step 5 — Tests

1. **`play_progress_bonus_fires_on_successful_play`** — assert non-zero
   bonus for a play that advances a stack.
2. **`play_progress_bonus_zero_on_misplay`** — assert zero for a play that
   doesn't advance (strike).
3. **`play_progress_bonus_zero_on_clue_or_discard`** — assert zero.
4. **`tempo_bonus_overrides_horizon_drag`** — synthetic two-line scenario
   where line A plays 2 cards and ends with pace=5, line B plays 0 cards
   and ends with pace=7. With `play_progress_weight = 1.0` and
   `pace_weight = 1.0`, A should beat B by 2 (plays) − 2 (pace delta) = 0
   on those terms, then any other positive signal on A wins. Or invert
   numbers so the test is unambiguous.
5. Run `tests/search_regression.rs` and check that the `avoid_stealing_finesse`
   scenario passes (the finesse line is now competitive even before Plan A
   is applied).

## Step 6 — Interaction with Plan A

Plan A removes the spurious `critical_in_hand` swing; Plan B counteracts the
remaining `pace` swing. They are complementary:

- With **A only:** the crit swing disappears, but pace still favours
  delayed plays. The finesse will gain ~+1.2 points; might still lose by
  ~0.0–0.2. Not robust.
- With **B only:** the crit swing remains but is offset by the tempo bonus.
  Works for *this* scenario but the underlying eval is still incoherent —
  expect new horizon-effect scenarios to appear in other places (early-game
  finesse-vs-stall, for instance).
- With **A + B:** the eval is coherent (Plan A) *and* tempo is rewarded
  explicitly (Plan B). This is the recommended end state.

If you ship in one PR, ship them together. If you ship in two PRs, ship A
first because B's tuning is easier when the underlying eval is no longer
biased.

## Estimated effort

- Step 1 (trait): 0.1 day
- Step 2 (default impl): 0.1 day
- Step 3 (wiring): 0.2 day
- Step 4 (magnitude pick): falls out of step 5
- Step 5 (tests): 0.5 day

**Total: ~1 day.**

---

## Suggested rollout order

1. **PR 1:** Plan B (tempo bonus). Ships cheap, makes the failing scenario
   pass, gives a clear win to point at.
2. **PR 2:** Plan A step 1 + step 2 (helper + per-term audit, no refactor).
   The bigger conceptual fix; tested against scenarios beyond
   `avoid_stealing_finesse`.
3. **PR 3:** Plan A step 3 (loop consolidation) + tuning + docs. Performance
   cleanup once correctness is established.

## Files touched

- `src/engine/evaluator.rs` — both plans.
- `src/engine/tree_action_selection_strategy.rs` — Plan B wiring.
- `src/game/deck.rs` — Plan A may want a helper here.
- `src/game/state/table_state.rs` — possibly, if anonymous-consumption
  tracking gets formalized (Plan A step 1a Option 2).
- `tests/search_regression.rs` — both plans expected to flip the failing
  scenario.
- `docs/architecture/design.md` and `docs/architecture/overview.md` — Plan A
  step 6.

## Out of scope

- Quiescence search / variable depth — discussed and explicitly rejected
  upstream of this plan; the eval fix obviates the need.
- Pace clamping / piecewise weights — a band-aid; not needed once Plan A is
  in.
- Re-tuning all weights from scratch — only the per-card weights need
  attention (step 5 of Plan A); the rest are stable.
