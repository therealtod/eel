# Search-Tree Visualizer — Implementation Plan

**Status:** Proposed (not implemented).
**Owner:** TBD.
**Goal:** Provide a CLI and library tool that, given a scenario, captures the engine's search rollout (root candidates, per-ply choices, pruned branches, leaf breakdowns) and renders it in human-readable form. Primary use case is debugging failures in `tests/search_regression.rs`.

## 1. Motivation

When a search regression fails (e.g. `tests/search_regression.rs::avoid_stealing_clue`), the assertion tells us only "engine chose X, expected Y". To diagnose this today, a developer must either:

- read code and reason from first principles about what the evaluator would prefer, or
- sprinkle `tracing` filters and `cargo test -- --nocapture` to see thousands of unstructured log lines.

Both are slow. The engine already emits structured `tracing` events at every ply (see `src/engine/tree_action_selection_strategy.rs:265-353, 473-484`); we just don't have a way to **harvest and render** them as a tree.

## 2. Goal

A `search-viz` binary that, given a scenario name, prints something like:

```
ROOT  P0  depth=6  4 candidates
├─ ✓ Clue rank=1 → P1 touches [9]   [DirectPlayClue]   total=23.45  (leaf=20.10 +Σimm=+3.35)
│  └─ P1  depth=5  3 candidates
│     ├─ ✓ Play deck#9                [PlayKnownPlayable] total=23.45
│     ├─   Discard deck#7             [DiscardChop]       total=19.10
│     └─ ✗ Clue color=B → P2          [DelayedPlayClue]   PRUNED  ceiling=22.50 ≤ best=23.45
├─   Clue color=R → P2                [DelayedPlayClue]   total=22.10
├─   Discard deck#3                   [DiscardChop]       total=15.00
└─ ✗ Play deck#2                      [BlindPlay]         PRUNED  ceiling=12.00 ≤ best=23.45

leaf breakdown (chosen line):
  stacks=20.00  clue_tokens=+0.50  bdr=-0.40  ...
```

## 3. Non-Goals

- No changes to evaluator math or convention logic.
- No new feature flags or runtime modes for production binaries.
- No GUI (an HTML output is Phase 3, but it is a static file — not an interactive app).
- Not a profiler. Wall-clock timing per node is out of scope.

## 4. Architecture

### 4.1 Capture strategy: tracing-subscriber Layer

The search hot path already emits the events we need. We do **not** instrument it further on the call side; instead we install a custom `tracing_subscriber::Layer` (call it `CaptureLayer`) only when the viz tool runs.

The relevant emission points in `src/engine/tree_action_selection_strategy.rs` (today):

| Site | Kind | Fields |
|---|---|---|
| `best_score_at_depth` entry | span `search_ply` | `depth`, `player`, `candidates` |
| Leaf branch | event `leaf_reached` | `depth`, `terminal`, `leaf` (Display) |
| Per-candidate eval | event `candidate_evaluated` | `action`, `tech`, `immediate`, `subtree_score`, `score`, `improved` |
| Per-candidate prune | event `candidate_pruned` | `action`, `tech`, `candidate_ceiling`, `best` |
| `scored_actions` root | span `scored_actions` | `player`, `candidates` |
| `scored_actions` per-root | event `candidate_scored` | `action`, `tech`, `priority`, `leaf_score`, `immediate_bonus`, `total`, `leaf`, `line` |

These are sufficient to reconstruct the full tree by span-id nesting.

### 4.2 Data model

New module `src/engine/search_viz/` (compiled into the main crate; gated only by the binary that uses it, not by a feature flag — it's pure data structures).

```rust
// src/engine/search_viz/tree.rs
pub struct SearchTree {
    pub root: PlyNode,
}

pub struct PlyNode {
    pub depth: usize,
    pub player: usize,
    pub candidates: Vec<CandidateNode>, // ordered as encountered; chosen flagged separately
    pub chosen_idx: Option<usize>,
}

pub struct CandidateNode {
    pub action: GameAction,
    pub tech_name: String,             // owned: tracing fields are recorded as Display
    pub immediate_bonus: f64,
    pub outcome: CandidateOutcome,
    pub child: Option<PlyNode>,        // None for leaves, pruned, or depth=0
}

pub enum CandidateOutcome {
    Scored { total: f64, leaf_breakdown: Option<ScoreBreakdown> },
    Pruned { ceiling: f64, best_at_prune: f64 },
}
```

`leaf_breakdown` is `Option` because today's `tracing::trace!` records it as `Display`; capturing the structured value requires a small enrichment (see §4.5).

### 4.3 CaptureLayer

```rust
// src/engine/search_viz/capture.rs
pub struct CaptureLayer { /* shared Arc<Mutex<Builder>> */ }

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for CaptureLayer { ... }
```

Implementation notes:

- Maintain a map `SpanId → PartialPlyNode` in span extensions (`tracing_subscriber::registry::SpanData::extensions_mut`).
- On `on_new_span` for `search_ply`/`scored_actions`: allocate a `PartialPlyNode`, attach to extensions.
- On `on_event` for `candidate_evaluated`/`candidate_pruned`/`leaf_reached`/`candidate_scored`: find the current span, push the event into its partial node.
- On `on_close`: finalize the partial node, attach it as the `child` of the parent span's last "in-flight" candidate (tracked via a small stack on the parent's extension).
- `rayon` parallelism at the root: each root candidate is evaluated on its own thread, but each gets its own `search_ply` span chain. The layer must key its bookkeeping per-span, not per-thread.

Output: `let tree: SearchTree = capture_layer.into_tree();` after the search completes.

### 4.4 Renderers

Three renderers, each a free function taking `&SearchTree` and writer:

- `render_text(tree, w, opts)` — ANSI-coloured tree. Default. Uses `termcolor` (already a transitive dep via clap-style ecosystem if added; otherwise plain). Colours: green = chosen, red = pruned, dim grey = sibling.
- `render_json(tree, w)` — `serde_json::to_writer_pretty`. `SearchTree` derives `Serialize`.
- `render_html(tree, w, opts)` — Phase 3. Static HTML with `<details>` collapsibles; no JS framework.

### 4.5 Tracing-event enrichment

Two small additions to `tree_action_selection_strategy.rs` to make capture lossless:

1. **`candidate_evaluated`** currently does not include the leaf breakdown for non-winning candidates. Add a `leaf = %leaf_bd` field. Cost: one `Display` call per candidate (cheap; trace level, already paid when subscriber is on).
2. **`leaf_reached`** records `leaf` as Display; add an extra `leaf_json` field via `tracing::field::display` of a small `ScoreBreakdownDisplay` wrapper that emits JSON. (Or: serialize via `tracing::field::valuable` once we adopt `valuable` — out of scope.) For Phase 1 we accept text-only leaf rendering and parse on the way out if needed; the structured breakdown is nice-to-have, not required.

These additions are zero-cost when no subscriber is installed.

### 4.6 CLI binary

```
src/bin/search_viz.rs
```

Argument parsing: `clap` (add as a dev/bin-only dependency). Flags:

| Flag | Description |
|---|---|
| `--scenario-name <NAME>` | Load `tests/scenarios/<NAME>/table_state.json` via the existing `tests/common` loader. |
| `--scenario-path <PATH>` | Direct path to a `table_state.json`. |
| `--format text\|json\|html` | Default `text`. |
| `--out <PATH>` | Write to file; otherwise stdout. |
| `--max-depth <N>` | Clip the printed tree depth (default: full). |
| `--top-k <K>` | At each ply show only top-K candidates by score (default: 4). |
| `--focus-action <SPEC>` | Only expand the branch matching this action (e.g. `clue=R@P1`, `play=4`). |
| `--focus-tech <NAME>` | Only expand branches whose root action was proposed by `<NAME>`. |
| `--show-leaves` | Print leaf breakdowns for *all* candidates, not just the chosen line. |
| `--no-prune` | Disable branch-and-bound for this run (see §6). |

Flow:

1. Parse args; resolve scenario.
2. Build `HGroupConventionSet` exactly as `tests/search_regression.rs::build_convention_set`. Lift that helper into `tests/common/` so it's shared.
3. Install `CaptureLayer` as the sole tracing subscriber for this process.
4. Build root POV; call `TreeActionSelectionStrategy::default().scored_actions(...)`.
5. Pull the tree out of the layer; render per `--format`.

### 4.7 Test ergonomics (Phase 2)

Add a macro in `tests/common/`:

```rust
assert_search_action!(
    scenario = "search/avoid_stealing_clue",
    matches: GameAction::Clue { player_index: 1, .. },
    description = "rank-1 clue to Bob"
);
```

On match it does nothing extra. On mismatch it installs `CaptureLayer`, re-runs `scored_actions`, prints the text tree to stderr, then panics with the original assertion message. This makes every search test self-debugging at zero cost on green runs.

## 5. File / Module Layout

```
src/engine/search_viz/
  mod.rs            # pub use
  tree.rs           # SearchTree, PlyNode, CandidateNode, CandidateOutcome
  capture.rs        # CaptureLayer + span-extension bookkeeping
  render_text.rs    # render_text + ANSI helpers
  render_json.rs    # render_json
  render_html.rs    # Phase 3
src/bin/
  search_viz.rs     # CLI entry point
tests/common/
  conventions.rs    # build_default_convention_set (lifted from search_regression.rs)
docs/development/
  search-visualizer-plan.md   # this document
  search-visualizer.md        # user-facing docs, written in Phase 1
```

## 6. Tradeoffs and Open Questions

- **Tracing-subscriber vs. instrumented search.** Subscriber wins on zero hot-path cost and a single source of truth. Cost: capture is coupled to event/span field names, which become an implicit contract. Mitigation: a unit test that runs a one-ply search with the capture layer and asserts the tree shape; if a future edit drops a field, the test breaks.
- **Pruned subtrees are invisible.** Branch-and-bound skips them before recursion. The tree can show *that* a candidate was pruned and the ceiling that pruned it, but not what its rollout would have scored. `--no-prune` is an opt-in escape hatch — implemented as a separate code path in `TreeActionSelectionStrategy` that takes a `prune: bool` flag, defaulting to `true`. Only used by the viz tool. Bounded slowdown (still depth-limited).
- **`rayon` root parallelism.** Spans are keyed by ID, so capture is correct, but emission order is non-deterministic. The renderer must sort root candidates by score before printing (which it already does to match `scored_actions` output).
- **Action formatting.** `decision_tree::fmt_action` already exists for `LineStep` Display. Reuse it; do not duplicate the formatting logic. If renderers want richer formats (e.g. mapping `card_deck_index` back to a colour/rank), extend that helper rather than fork it.
- **Leaf breakdown structured capture.** Phase 1 keeps `ScoreBreakdown` as `String` in the tree (recovered from the Display field). Phase 2 may upgrade to structured capture (see §4.5 option 2) if renderers need to slice by term.
- **Convention-set provenance.** The viz tool hardcodes `HGroupConventionSet` for now. If/when other convention sets land, accept `--conventions hgroup|refsieve|…`.

## 7. Phasing

### Phase 1 — MVP (~½ day)
- [ ] `SearchTree` / `PlyNode` / `CandidateNode` data model (`src/engine/search_viz/tree.rs`).
- [ ] `CaptureLayer` covering `search_ply`, `scored_actions`, `candidate_evaluated`, `candidate_pruned`, `leaf_reached`, `candidate_scored`.
- [ ] `render_text` with ANSI colours; chosen path green, pruned red, others dim.
- [ ] `src/bin/search_viz.rs` with `--scenario-name`, `--format text` only, full-depth.
- [ ] Lift `build_convention_set` from `tests/search_regression.rs` to `tests/common/`.
- [ ] Smoke test: a unit test that runs the capture layer on `search/play_known_playable` and asserts root has ≥1 candidate with `Scored` outcome.
- [ ] `docs/development/search-visualizer.md` — usage docs.

### Phase 2 — Filters & test integration (~½ day)
- [ ] JSON renderer.
- [ ] `--max-depth`, `--top-k`, `--focus-action`, `--focus-tech`, `--show-leaves`.
- [ ] `--no-prune` mode (adds a `prune: bool` knob to `TreeActionSelectionStrategy::scored_actions_with_options`).
- [ ] `assert_search_action!` macro in `tests/common/`; retrofit at least one existing search test as a worked example.
- [ ] Structured `leaf_breakdown` capture (event enrichment).

### Phase 3 — Optional polish
- [ ] HTML renderer (single static file, `<details>`-based).
- [ ] Cross-scenario diff: `--diff-against <other-scenario>` printing side-by-side or unified-diff-style.
- [ ] Action-formatting upgrade: map `card_deck_index` → "R1 (deck#9)" using `StaticGameData` + `Deck`.

## 8. Acceptance Criteria

- `cargo run --bin search-viz -- --scenario-name search/avoid_stealing_clue` prints a tree whose top-ranked candidate matches what `cargo test avoid_stealing_clue` says the engine chose.
- Disabling the capture layer (i.e. running normal `cargo test`) shows no measurable change in `scored_actions` wall time (within noise on a release build).
- Smoke test passes on CI.
- Phase 2: A search regression test annotated with `assert_search_action!` produces a readable tree on stderr when it fails.

## 9. References

- `src/engine/tree_action_selection_strategy.rs` — search loop and existing tracing.
- `src/engine/decision_tree.rs` — `ScoredNode`, `LineStep`, `fmt_action`.
- `src/engine/evaluator.rs` — `ScoreBreakdown` (consider deriving `Serialize`).
- `tests/search_regression.rs` — current test format and convention-set construction.
- `tests/common/` — scenario loader (`load_scenario_by_name_with_knowledge`).
