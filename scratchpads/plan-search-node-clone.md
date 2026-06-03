# Implementation Plan — Stop cloning `StaticGameData` (and `history`) into search nodes

Addresses improvements.md item #1.

## Problem restated (with one correction)

`KnowledgeAwareGameState` derives `Clone` and is the unit cloned per-ply in search
(`tree_action_selection_strategy.rs:548, 845, 899, 1110, 1372`). Beyond the intended cheap
`table_state` / `team_knowledge`, it owns two fields that should not be copied per node:

- **`static_data: StaticGameData`** (`:111`) — directly violates AGENTS.md ("StaticGameData
  should not be cloned into search nodes"). `Variant` is the bulk of it: `empathy_by_clue`
  `[[u64;6];2]` (96 B) + `card_copies_count_by_id`/`rank_by_id` `[u8;35]` ×2 + assorted
  bitfields ≈ ~200 B of pure config copied on every `state.clone()`. It is **read-only** — no
  method mutates it.
- **`history: Vec<GameStateSnapshot>`** (`:117`) — *intended* to be the expensive part (each
  snapshot is a full `TableState` + `TeamKnowledge`).

**Correction to the scratchpad premise:** `history` is in practice **always empty**.
`record_snapshot` (the only pusher, `:1053`) and `pov_at_turn` (`:1064`) and `history_len`
(`:1074`) have **zero call sites** across `src/` and `tests/`. The play/discard apply paths
pass `&self.history` (empty) to `collect_hypotheses` (`:392`, discard equivalent); the clue path
builds its own `local_history` from a fresh snapshot (`:674–704`) and ignores `self.history`
entirely. So today the history *clone* is a 24-byte empty-Vec copy, not a deep copy — the real,
measurable waste is `static_data`. The plan removes the latent footgun anyway (a future
`record_snapshot` caller would silently reintroduce per-ply deep copies) but we should not
claim a history speedup we won't get.

## Step 0 — Measure first (required by `docs/rust/style.md`: "Benchmark before micro-optimizing")

There is no `benches/` harness yet (`Cargo.toml` only declares the `test-support` feature).
Establish a number before and after:

1. Add a Criterion bench (or a `#[ignore]` timing test behind `test-support`) that drives
   `scored_actions` on a representative mid-game `PlayerPOV` fixture for a couple of variants
   (2p and 5p, since clone cost scales with player count via `TeamKnowledge`).
2. Record ns/iter and, optionally, `std::mem::size_of::<KnowledgeAwareGameState>()` and the
   `StaticGameData` byte count, to confirm the share attributable to `static_data`.
3. Keep the bench; it guards items #3 (`apply_clue` table clone) and the broader search work.

If the measured `static_data` share is negligible, stop here and downgrade the item — but the
AGENTS.md constraint is normative regardless of magnitude, so a borrow-based fix is still
justified as correctness-of-design even if the speedup is small.

## Step 1 — Decide the split (recommended: borrow via lifetime)

Goal (scratchpad's own recommendation): "split the cloned search node from the borrowed session
context (`static_data`, `history`)." Three options:

- **A. Lifetime-parameterized struct (recommended).** `KnowledgeAwareGameState<'a>` holding
  `static_data: &'a StaticGameData` and `history: &'a [GameStateSnapshot]`. The owned, genuinely
  per-node fields (`table_state`, `team_knowledge`, `next_deck_index`, `next_hypothesis_id`,
  `phantom_claimed_mask`) still `Clone` cheaply; the two borrows are `Copy`, so the derived
  `Clone` now copies *only* mutating state. Aligns with style.md ("Prefer borrowing over
  cloning") and AGENTS.md ("immutable snapshots for branching"). Cost: thread `<'a>`/`<'_>`
  through type mentions (counts below).
- **B. Pass `static_data`/`history` as parameters** to `apply` and friends instead of storing
  them. More churn — `static_data` is referenced internally in ~40 spots; every helper would
  grow a parameter. Rejected.
- **C. `Arc<StaticGameData>`.** Smallest diff (field type + `Arc::clone` is a refcount bump,
  not a data copy). style.md only forbids `Arc<Mutex>`, so plain `Arc` is permissible, but it's
  a runtime refcount where a compile-time borrow is available, and it muddies "should not be
  cloned into search nodes." Keep as fallback if the lifetime threading proves disproportionate.

**Recommendation: Option A.** The borrow lifetime is naturally available: every search entry
point already obtains `static_data` as `&StaticGameData` from the POV
(`scored_actions:1333`, `replay_leaf_breakdown:1289`) which outlives the whole rollout, so the
root and all descendant clones can borrow it.

## Step 2 — Refactor `KnowledgeAwareGameState` (Option A)

In `src/engine/knowledge_aware_game_state.rs`:

1. Add lifetime: `pub struct KnowledgeAwareGameState<'a> { … }` with
   `static_data: &'a StaticGameData` and `history: &'a [GameStateSnapshot]`.
2. Replace every `&self.static_data` with `self.static_data` (already a reference) — mechanical;
   `static_data()` accessor (`:938`) already returns `&StaticGameData` and is unchanged.
3. Constructors:
   - `from_parts` (`:153`) takes `static_data: &'a StaticGameData` and sets
     `history: &[]` (empty slice literal). Update the two search callers (`:1294`, `:1338`) and
     `replay/from_scenario.rs:126` to drop their `.clone()` and pass `&static_data`.
   - `new` (`:137`) is used only by `replay/reconstruct.rs` (`:102`, `:129`). Two sub-options:
     keep an owning replay path (see Step 3) or have `new` also borrow. Decide in Step 3.
4. `record_snapshot` / `snapshot` / `pov_at_turn` / `history_len`: since `history` is now a
   borrow, these can't push. They are **dead** (no callers). Preferred: **delete them** and the
   `history` field's mutability, keeping `history: &'a [GameStateSnapshot]` purely as the
   read-only slice threaded into `collect_hypotheses`. This is the minimal-surface change and
   removes the "silently reintroduces deep copies" footgun. Update the struct doc comment
   (`:104–106`) which currently advertises the now-removed snapshot/history workflow.
5. `apply` and the clue path keep working unchanged: `collect_hypotheses(..., self.history, ...)`
   passes the borrowed (empty) slice; `apply_clue`'s `local_history` is unaffected.

## Step 3 — Reconcile the replay reconstruct path

`replay/reconstruct.rs` builds via `KnowledgeAwareGameState::new(static_data.clone())` and
mutates across turns. It does **not** call `record_snapshot` either, so it needs no history
accumulation. Make it hold the `StaticGameData` in the surrounding driver struct and pass
`&static_data` into a borrowing `new`/`from_parts`. Verify the reconstruct struct can own the
`StaticGameData` alongside the `KnowledgeAwareGameState<'a>` without a self-referential-struct
problem — if the game state and its `static_data` owner would live in the same struct, store the
`StaticGameData` one level up (in the caller that constructs the reconstructor) and pass the
borrow in. This is the one place to watch for lifetime friction; resolve before touching tests.

## Step 4 — Thread the lifetime through signatures

Add `<'_>` (or a named lifetime where a borrow is returned/stored) to bare `KnowledgeAwareGameState`
mentions. Approximate sites to update (from grep):

- `tree_action_selection_strategy.rs` — ~10 `&KnowledgeAwareGameState` params (`:225, 247, 248,
  416, 663, 685, 729, 746, 834, 984, 1035, 1070`) → `&KnowledgeAwareGameState<'_>`.
- `evaluator/types.rs` (13), `evaluator/scoring.rs` (12), `play_resolver.rs` (9),
  `replay/from_scenario.rs` (5), `replay/snapshot.rs` (2) — mostly `&KnowledgeAwareGameState`
  receivers/params; add `<'_>`.
- `evaluator/tests.rs` — ~15 `from_parts(static_data, …)` call sites become
  `from_parts(&static_data, …)`; the `static_data` binding must outlive the returned state
  (it already does in each test fn). A few helper fns that *return* a `KnowledgeAwareGameState`
  (e.g. `:68, 401, 748, 1147`) currently move `static_data` out — those must instead take/return
  with a lifetime tied to a `static_data` the caller owns. Adjust those helpers to accept
  `&'a StaticGameData` and return `KnowledgeAwareGameState<'a>`.

The test-helper return cases are the only non-mechanical edits in this step.

## Step 5 — Validate

- `cargo build` then `cargo clippy` — fix lifetime elision warnings.
- `cargo test` and `cargo test --test '*'` — replay regression and search regression must stay
  green (no behavior change is intended; this is a representation change).
- Re-run the Step 0 bench; record the delta. Expectation: clone cost drops by the
  `StaticGameData` share; correctness/scores identical.

## Step 6 — Docs

- `docs/architecture/overview.md:15` describes `KnowledgeAwareGameState` and the snapshot/history
  workflow; update it to reflect that `static_data`/`history` are borrowed session context and
  that the snapshot helpers were removed (also resolves improvements.md item #10's "WIP wrapper"
  wording while we're here, if desired).
- Update the struct-level doc comment in `knowledge_aware_game_state.rs` (Step 2.4).
- Note the change in `AGENTS.md` only if a constraint's wording needs it (it already mandates the
  split — no edit expected).

## Risks / notes

- **Self-referential struct** in the replay reconstructor (Step 3) is the main lifetime hazard;
  keep `StaticGameData` owned by the caller, not co-located with the borrowing state.
- If lifetime threading balloons beyond the sites above, fall back to **Option C (`Arc`)** for a
  smaller diff that still eliminates the data copy. Decide at the end of Step 4 based on actual
  churn, not upfront.
- Scope guard (AGENTS.md "minimal diffs"): do **not** fold in items #3 (apply_clue table clone)
  or #4 (convention filtering in search) here; this change is representation-only.
