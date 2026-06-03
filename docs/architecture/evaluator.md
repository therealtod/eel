# Evaluator — Term Reference

`DefaultEvaluator` (defined in `src/engine/evaluator/`) scores every leaf state reached
during alpha-beta search.  The total score that drives action selection is:

```
total_score = leaf_total + Σ immediate_bonus(ply)
```

- **`leaf_total`** — the sum of terms measured once at the final state of a search line.
- **`Σ immediate_bonus`** — per-action bonuses/penalties accumulated at every ply along
  the line (see `LineStep.immediate_bonus` in `decision_tree.rs`).

`ScoreBreakdown` exposes each leaf term individually and reserves three zero-at-leaf
fields (`team_empathy`, `critical_exposure_delta`, `tempo_cost`) so callers can optionally
fold the accumulated per-ply contributions in for display.

---

## 1 — Leaf Terms

Evaluated once on the state at the end of the search line.  Higher is always better.

### 1.1 `game_score`

```
score_weight × Hanabi_score(state)
```

The game's actual score (sum of playing-stack tops), counting any phantom plays
accumulated along the search path.  This is the primary objective and deliberately
dominates the total: `score_weight = 10.0` by default, so every extra card played is
worth 10 raw points.

**Sign:** `+`  
**Default weight:** `score_weight = 10.0`

---

### 1.2 `strike_penalty`

```
strike_penalties[strike_count]
```

A look-up into `[0.0, 10.0, 30.0, 1000.0]` indexed by the number of strikes taken.
The 3-strike (game-over) penalty is set high enough that any in-progress game always
scores higher than a struck-out one.  The non-linear spacing (0 → 10 → 30 → 1000)
makes the engine strongly risk-averse near the strike limit.

**Sign:** `−` (subtracted from total)  
**Default values:** `[0.0, 10.0, 30.0, 1000.0]`

---

### 1.3 `pace`

```
pace_weight × clamp(pace, −10, num_players)
```

Where:

```
pace = score + cards_in_deck + num_players − max_theoretical_score
```

**Pace is the number of discards the team can still afford without lowering the score
ceiling.**  Each discard permanently consumes a turn that could have been used to play a
card; with pace = 0 the team cannot discard at all and must play (or give a useful clue)
on every turn to avoid missing the maximum score.  Pace −n means the maximum achievable
score is automatically reduced by n regardless of future play — the game has already lost
n cards' worth of turns.

Pace is also a measure of how many players are allowed to hold hands with no
eventually-playable card at any point: pace 0 means every card in every hand must
eventually be played.

The `clamp(·, −10, num_players)` keeps catastrophic states from distorting search and
prevents the engine from being complacent when the deck is large.

**Sign:** `+`  
**Default weight:** `pace_weight = 1.0`

---

### 1.4 `efficiency_penalty`

```
efficiency_weight × required_efficiency
```

Where:

```
required_efficiency = max(0, still_to_play − live_setups) / spare_turns
```

- `still_to_play` = cards remaining to be played to reach max score.
- `live_setups` = clue-touched cards still in hands (already staged; each reduces the
  number of fresh clues still needed).
- `spare_turns` = `cards_remaining − still_to_play` — the turns available beyond the bare
  minimum to play every remaining card (i.e. the discard budget, equivalent to pace).

**Required efficiency is the average number of cards that each clue must eventually get
played, to finish the game.**  A value of 0.5 means the team has enough slack to spend
2 clues per 1 card played; a value of 1.5 means every 2 clues must generate 3 plays —
multi-card setups such as finesses or colour clues touching several playables become
necessary.  Values above 1 indicate real clue pressure; values near or above 2 are
typically losing positions.

The term penalises states where the search path has burned turns without setting up
plays, driving up the required efficiency the team must deliver for the rest of the game.

**Sign:** `−`  
**Default weight:** `efficiency_weight = 1.9`

---

### 1.5 `critical_exposure_penalty`

```
critical_exposure_weight × Σ threat(card)
```

For every truth-critical card held untouched in a player's hand:

```
threat = 1 / (1 + rank_from_chop + 4 × buffer_count)
```

- `rank_from_chop` — 0 for the current chop slot, 1 for the next discard-eligible slot,
  etc. (only untouched, non-known-trash cards are discard-eligible).
- `buffer_count` — count of known-playable **or** known-trash cards in the holder's hand.
  Each one represents an action (play or trash-discard) the holder will take before being
  forced to discard the critical card.

Buffer dominates position (factor 4): one buffer action is roughly equivalent to moving
the card four slots away from chop, so even a rightmost-but-buffered critical is much
safer than an untouched-and-unbuffered slot-1 critical.

Touched non-playable cards (saves, prompts) are held indefinitely and contribute zero —
they are neither buffer nor discard-eligible.  Cards without a known truth identity (fresh
draws in search) also contribute zero; probabilistic risk from those belongs to BDR
(§2.2).

**Sign:** `−`  
**Default weight:** `critical_exposure_weight = 3.0`

---

### 1.6 `lost_ceiling_penalty`

```
lost_score_ceiling_weight × (theoretical_max − max_achievable_score)
```

`max_achievable_score` walks each suit's unplayed ranks and stops at the first rank where
all copies have been discarded (that rank and everything above is permanently lost), also
capping by pace.  Any gap between the theoretical maximum and what is still achievable is
penalised here.  This makes the engine strongly avoid the last discard of any needed card.

**Sign:** `−`  
**Default weight:** `lost_score_ceiling_weight = 8.0`

---

### 1.7 `empathy_bonus`

```
empathy_weight × Σ (max_identities − popcount(effective_mask))
             over all clued cards in every player's hand
```

Rewards narrower inferred identity ranges on clued cards.  A card fully resolved to one
identity contributes `max_identities − 1`; a card with all identities open contributes 0.
Uses the combined (game-rule + convention-inferred) identity mask.

Disabled by default (`empathy_weight = 0.0`) because `team_empathy_delta_bonus` (§2.1)
captures the same information without the draw-dilution bias introduced by a static leaf
measurement.

**Sign:** `+`  
**Default weight:** `empathy_weight = 0.0`

---

### 1.8 `clue_tokens`

```
clue_token_weight × H(n) × (1 + clue_demand_weight × demand)
```

Where:

- `H(n) = 1 + 1/2 + 1/3 + … + 1/n` — harmonic number for `n` whole clue tokens.  The
  0 → 1 jump is the largest; each additional token adds diminishing marginal value,
  reflecting that a dry clue bank is catastrophically bad, while surplus tokens have low
  incremental worth.
- `demand` = Σ `max(P(card is playable), P(card is critical))` over all unclued hand cards,
  using each card's global empathy.  High demand amplifies the token value: clues are more
  urgent when hands hold many unknown playables or criticals.

**Sign:** `+`  
**Default weights:** `clue_token_weight = 0.6`, `clue_demand_weight = 0.05`

---

### 1.9 `known_playable`

```
known_playable_weight × count(cards any player knows are playable)
```

A card counts if:

1. It carries a `Signal::Play` (convention blind-play signal), **or**
2. Its convention-inferred identity set is non-empty and wholly inside the playable mask,
   **or**
3. Its game-rule empathy set is non-empty and wholly inside the playable mask.

Truth-guarded: even if the holder thinks the card is playable, if the searcher's truth
view shows it is not, it is excluded.  This prevents rewarding clues that set up a
misinformed play.

**Sign:** `+`  
**Default weight:** `known_playable_weight = 1.0`

---

### 1.10 `misinformation_penalty`

```
misinformation_weight × Σ penalty(card)
```

Three-case formula per own-hand card with a known truth:

| Case | Penalty |
|------|---------|
| `effective_mask` is a singleton equal to truth (exact knowledge) | 0 |
| Truth is in `effective_mask` and `effective_mask` has `n > 1` identities | `(n − 1) / n` |
| Truth is excluded from `effective_mask` (committed to wrong identity) | 1.0 (full) |

Models the cost of a convention breakdown — a player reading a card incorrectly will
either bomb it or fail to play it when needed.  Disabled by default
(`misinformation_weight = 0.0`).

**Sign:** `−`  
**Default weight:** `misinformation_weight = 0.0`

---

## 2 — Immediate Bonuses / Penalties

These are computed per action as the search walks each ply and accumulated into
`LineStep.immediate_bonus`.  They are **not** part of the leaf state evaluation.

`total_score = leaf_total + Σ immediate_bonus(ply)`

---

### 2.1 `team_empathy_delta_bonus` (clue actions only)

```
team_empathy_weight × (team_empathy_score_after − team_empathy_score_before)
```

Where `team_empathy_score = Σ (max_ids − popcount) / max_ids` across all own-hand cards
for all players (using game-rule empathy from `Deck`).

Fires only on `GameAction::Clue`.  Measuring the delta rather than the absolute value
avoids the **draw-dilution bias**: a freshly drawn card has near-maximum uncertainty and
would lower the sum, making any line with an extra play (= extra draw) look worse on this
term even when it strictly improved the game state.

**Sign:** `+` (positive when clue tightens team identity)  
**Default weight:** `team_empathy_weight = 0.3`

---

### 2.2 `critical_exposure_delta_bonus`

```
±critical_exposure_delta_weight × threat
```

For each teammate whose **chop** holds a truth-critical card before the action:

- **`+weight × threat`** if the action clue-touched the chop card (save reward).
- **`−weight × threat`** if the card remains unsaved **and** the actor had at least one
  clue token (opportunity to save was available but was not taken).

The penalty for skipping a save is only charged when no intermediate player (between actor
and chop owner in turn order) has *fewer* known-playables than the actor — that player
would be a more natural saver, so the actor is not primarily responsible.

`threat = 1 / (1 + 4 × buffer)` mirrors the leaf's `critical_exposure_score` formula
(buffer = known-playable or known-trash cards in the chop owner's hand), keeping the
immediate incentive proportional to actual discard risk.

**Sign:** `+` for saves, `−` for skip  
**Default weight:** `critical_exposure_delta_weight = 0.5`

---

### 2.3 `tempo_cost`

```
−tempo_cost_weight   per non-progress action
```

Charged on every action that does **not** advance the play stacks:

| Action | Tempo cost |
|--------|-----------|
| Successful `Play` (score advances) | 0 — reward goes to `play_progress_bonus` |
| `Discard` | 0 — tempo-neutral; risk priced by `discard_action_penalty` |
| `Clue` | `−tempo_cost_weight` |
| Misplay (Play without score advance) | `−tempo_cost_weight` |

This closes the **horizon-arbitrage hole**: without it, the engine could pad a search line
with redundant clues to push an inevitable discard past the leaf horizon and avoid the
leaf's efficiency penalty.  Every padding clue now costs `tempo_cost_weight`, so the
engine cannot earn a free advantage by deferring bad events.  The weight should be at
least as large as the spurious gain from postponing a typical discard by one ply (≈ 1.0–3.0
at default weights).

**Sign:** `−`  
**Default weight:** `tempo_cost_weight = 2.5`

---

### 2.4 `clue_precision_bonus` (clue actions only)

Three sub-terms, all fired once per `GameAction::Clue`:

#### 2.4a Precision reward

```
+clue_precision_weight × count(touched cards resolved to a single identity after the clue)
```

Fires on each touched card that is `combined_possible_identities.is_exactly_known()` in
the post-clue state.

**Default weight:** `clue_precision_weight = 0.4`

#### 2.4b Good-touch penalty

```
−good_touch_penalty × bad_touch_count
```

`bad_touch_count` = number of touched cards whose truth identity is already played on the
stacks, already in the discard pile, or otherwise no longer needed — a violation of the
good-touch principle.  Should be lower than `lost_score_ceiling_weight` so that ceiling
preservation takes precedence over avoiding bad touches.

**Default weight:** `good_touch_penalty = 25.0`

#### 2.4c Potential bad-touch penalty

```
−potential_bad_touch_penalty   (flat, fires at most once per clue)
```

Fires when the OR of touched-on-receiver truth identities intersects the OR of inferred
identities of the giver's own-hand cards that are themselves touched or play-signalled
(after filtering out trash from both sides).  Models the risk of giving a clue that
duplicates a card the giver already knows they hold.

**Default weight:** `potential_bad_touch_penalty = 9.0`

---

### 2.5 `signal_ignored_penalty`

```
−signal_ignored_penalty_weight
```

Charged when the actor takes any action other than playing the specific card that carries
an active `Signal::Play`, and the signal is on an **untouched** card.  (Touched cards with
a play signal are handled by `PlayKnownPlayable`, not the urgency rule.)

Models the H-Group convention that a finessed player **must** resolve the blind-play on
their very next turn — stalling corrupts downstream interpretations.

**Sign:** `−`  
**Default weight:** `signal_ignored_penalty_weight = 5.0`

---

### 2.6 `play_progress_bonus`

```
+play_progress_weight   when a Play action advances the score
```

Fires on `GameAction::Play` when the post-action score (including phantom plays) is
strictly greater than the pre-action score.  Misplays (score unchanged) earn nothing;
the strike penalty at the leaf already penalises them.

Counteracts the structural **draw-dilution bias**: when a play draws a fresh card that
card has near-maximum empathy uncertainty, which slightly lowers leaf terms like
`clue_tokens` and `known_playable`.  Setting `play_progress_weight = 1.0` exactly
counteracts this drag; the current default of `2.0` adds a slight extra incentive to
play.

**Sign:** `+`  
**Default weight:** `play_progress_weight = 2.0`

#### Deferred-play discount

```
play_progress_bonus −= play_deferral_weight × elapsed_plies   (elapsed = number_of_players × 2 − remaining_depth)
```

A successful play loses a small amount for every ply it is deferred from the root, so an
**earlier play strictly beats an identical deferred one**.  This closes a second
horizon-arbitrage hole distinct from `tempo_cost`: when two lines play the *same* cards and
reach the *same* horizon-capped `game_score`, the line that postpones its plays banks extra
leaf `known_playable` credit for cards left un-cashed in hand, out-scoring the line that
cashed its progress efficiently.  Because the discount only ever *reduces* the immediate
bonus (never below zero given `play_deferral_weight × max_elapsed ≪ play_progress_weight`),
search `upper_bound` admissibility is preserved.

**Default weight:** `play_deferral_weight = 0.15`

#### Deferred-score discount (quiescence tail only)

```
game_score contribution −= score_deferral_weight × elapsed_plies   (continued past the leaf into the quiescence tail)
```

The deferred-play discount above only bites on the small `play_progress_bonus`; the dominant
`score_weight` reward a play realizes is undiscounted. That is harmless **inside** the horizon —
by the leaf, an early and a late in-horizon play have both landed in `game_score` identically —
but breaks down in the **quiescence tail** (`quiescence-search`), which cashes pending plays into
real `game_score` *beyond* the horizon. There, a tail play banks full `score_weight`, so a line
that **manufactures** extra tail-cashable plays (e.g. cluing more cards playable) out-scores one
that simply cashes a guaranteed play now — the horizon arbitrage relocated past the leaf. The
deferred-score discount decays the realized score per deferred ply, restoring "cash now > set up
for later". Like the play discount it only *reduces* leaf values, so `upper_bound` admissibility
is preserved. Consumed only by `quiescence_rollout`; inert when `quiescence-search` is off.

**Sign:** `−`  
**Default weight:** `score_deferral_weight = 0.75`

#### Declined-play debt (quiescence tail only)

The deferred-score discount above is a *flat* per-ply decay — it narrows the deferral
arbitrage but does not eliminate its sharpest form: a player who holds a **guaranteed** play
and *discards/clues instead* still has that play cashed for free past the horizon, so the line
banks both the discard's value (a clue token, a saved chop) *and* the deferred play's score.
"Cash now" then only weakly beats — or even loses to — "discard now, cash later".

The debt mechanism closes this at its source. Along the root truth simulation (in-horizon path
*and* the root action itself), each time a player takes a non-`Play` action while
`player_has_uncashed_signalled_play` holds for them, a per-player **declined-play debt** is
incremented (`[u8; MAX_PLAYERS_IN_GAME]`, threaded through `simulate_forward` →
`quiescence_extend` → `quiescence_rollout`). When the quiescence tail later cashes a play for a
player carrying debt, that play's full realized score is neutralized:

```
play_bonus = −play_score_weight − play_deferral_penalty(elapsed) − score_deferral_penalty(elapsed)
```

The first term exactly cancels the `score_weight` the tail play folds into the leaf (so
re-cashing the declined play banks *none* of it); the play then pays the same deferral discount
as any other tail play (no exemption for the wasted turn), landing it net-negative. Deferring a
play you could have made *now* is thus strictly worse than making it. Like the discounts above
it only *reduces* leaf values, so `upper_bound` admissibility is preserved. Inert when
`quiescence-search` is off, and exposed via the `Evaluator::play_score_weight` hook
(`DefaultEvaluator` returns `score_weight`).

**Root-playability repay gate.** The debt is a per-player counter, not card-specific, so the
neutralization is gated on the *cashed* card being a **pre-existing play** — one already playable
in the *root* state. The set of root-playable identities is threaded through `simulate_forward` →
`quiescence_extend` → `quiescence_rollout`, and `cashed_play_is_root_playable` checks the acting
player's own `Signal::Play` committed identity (or inferred set ⊆ root-playable) for the card being
cashed. A play the line itself unlocks — a finesse payoff that lands one ply past the horizon
purely from turn order — is **not** root-playable, so it banks its score normally instead of
repaying the debt.

Without this gate, demonstrating a finesse reads as deferral arbitrage: a teammate continues the
finesse with a clue while *also* holding a pre-existing play (incrementing the counter), cashes the
pre-existing play in-horizon, but the lingering debt is then wrongly repaid by — and wipes the score
of — the unrelated, line-unlocked payoff cashed in the tail. The engine would refuse to demonstrate
finesses (regression test `should_prioritize_demonstrating_finesse`).

---

### 2.7 `discard_action_penalty`

Two sub-terms, both fired once per `GameAction::Discard`:

#### 2.7a Bottom Deck Risk (BDR)

```
−bottom_deck_risk_weight × Σ P(card_is_id) × P(surviving_copy_lost) × loss_if_lost(id)
```

For each candidate identity `id` in the discarded card's empathy (or the singleton truth
if visible to the searcher):

- **Term A** `P(card_is_id) = 1 / popcount` — uniform over empathy.
- **Term B** `P(surviving_copy_lost) = 1 / max(1, deck_remaining)` — Laplace bottom-card
  probability; small with a full deck, approaches 1 as the deck empties.
- **Term C** `loss_if_lost` = count of still-achievable ranks from `rank_idx` upward (stops
  at the first rank where all copies are already discarded).

Skips terminal-rank identities (single copy by construction) and identities whose
surviving copy is visible in another player's hand.  Handles the **probabilistic** critical
risk from cards of unknown identity; the leaf's `critical_exposure_penalty` handles the
**known** critical risk.

**Default weight:** `bottom_deck_risk_weight = 2.0`

#### 2.7b Discard while known-playable

```
−discard_while_known_playable_penalty   (flat)
```

Charged when the actor discards while they hold at least one card that the search's play
generators would offer them (a `Signal::Play` card, or empathy wholly inside the playable
mask).  Models the H-Group expectation that a player with a known-playable plays it — a
discard instead implies the team did not save a critical onto chop, so the chop being
dumped may itself be critical.

**Default weight:** `discard_while_known_playable_penalty = 8.0`

---

## 3 — Weight Reference

| Field | Default           | Sign | Category |
|-------|-------------------|------|----------|
| `score_weight` | 10.0              | + | Leaf |
| `strike_penalties` | [0, 10, 30, 1000] | − | Leaf |
| `pace_weight` | 1.0               | + | Leaf |
| `efficiency_weight` | 1.9               | − | Leaf |
| `critical_exposure_weight` | 3.0               | − | Leaf |
| `lost_score_ceiling_weight` | 8.0               | − | Leaf |
| `empathy_weight` | 0.0               | + | Leaf (disabled) |
| `clue_token_weight` | 0.6               | + | Leaf |
| `clue_demand_weight` | 0.05              | + (amplifier) | Leaf |
| `known_playable_weight` | 1.0               | + | Leaf |
| `misinformation_weight` | 0.0               | − | Leaf (disabled) |
| `team_empathy_weight` | 0.3               | + | Immediate (clue Δ) |
| `critical_exposure_delta_weight` | 0.5               | ± | Immediate (per turn) |
| `tempo_cost_weight` | 2.5               | − | Immediate (per turn) |
| `clue_precision_weight` | 0.4               | + | Immediate (clue) |
| `good_touch_penalty` | 25.0              | − | Immediate (clue) |
| `potential_bad_touch_penalty` | 9.0               | − | Immediate (clue) |
| `signal_ignored_penalty_weight` | 5.0               | − | Immediate (any) |
| `play_progress_weight` | 2.0               | + | Immediate (play) |
| `play_deferral_weight` | 0.15              | − | Immediate (play, per elapsed ply) |
| `score_deferral_weight` | 0.75              | − | Quiescence tail (play, per elapsed ply) |
| `bottom_deck_risk_weight` | 2.0               | − | Immediate (discard) |
| `discard_while_known_playable_penalty` | 8.0               | − | Immediate (discard) |

---

## 4 — Design Notes

### Leaf vs. immediate: the draw-dilution problem

Several terms that were originally leaf measurements have been moved to immediate deltas:

- **`team_empathy`**: a static leaf score summing `(max_ids − popcount) / max_ids` over all
  hand cards is lowered by every fresh card drawn (maximum uncertainty), creating a bias
  against plays.  Measuring the *change* triggered by a clue, and crediting it immediately,
  captures the information gain without taxing other action types.

- **`critical_exposure_delta`**: rewarding/penalising per-turn changes in critical-chop
  situations provides a local, timely incentive.  Using only the leaf's
  `critical_exposure_penalty` would miss situations where the engine delays saving past the
  search horizon.

- **`play_progress_bonus`**: even with `game_score` at the leaf, the slight leaf-side drag
  from drawing a fresh card (lowering `clue_tokens`, `known_playable`, etc.) can bias the
  engine towards clues.  Crediting the progress immediately restores neutrality.

### Horizon arbitrage and `tempo_cost`

Without a tempo cost, the engine could improve a line's score by inserting extra clues
between an inevitable discard and the leaf: each added ply shifts the leaf evaluation
forward in time, potentially showing a better pace or efficiency snapshot.  Every clue
now pays `−tempo_cost_weight`, closing this loophole.  Discards are exempt because their
cost is already priced by `discard_action_penalty`.

### `critical_exposure_penalty` vs. `bottom_deck_risk`

These two terms are complementary:

- `critical_exposure_penalty` fires when the searcher's truth view **confirms** a card in
  hand is critical — certain knowledge of an impending danger.
- `bottom_deck_risk` fires on discards where the card's identity is **uncertain** —
  expected-value risk that the discarded card could have been the last copy of something
  needed.

Neither duplicates the other; together they cover the full spectrum from confirmed
known-critical exposure to speculative future-ceiling risk.

### Branch-and-Bound pruning via `upper_bound`

`DefaultEvaluator` implements `upper_bound(table_state, static_data, depth)` returning an
optimistic ceiling on the total score achievable from `table_state` with `depth` more plies,
combining maximum possible leaf terms with maximum possible per-ply immediate bonuses.
The tree uses this to prune branches where the best possible outcome is already beaten by a
known candidate.
