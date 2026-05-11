# Project Overview

**Eel** is a high-performance Hanabi card game bot written in Rust. It is designed around alpha-beta search (similar to
a chess engine) and uses bitmaps extensively for fast state computation.

# Eel Agent Instructions

Always read:
- `docs/architecture/overview.md` — module map and component descriptions
- `docs/rust/style.md` — coding standards

Load only when relevant to the task:

| Doc | Load when… |
|-----|------------|
| `docs/architecture/design.md` | Making structural or architectural decisions |
| `docs/development/testing.md` | Writing or debugging tests, or working with scenario JSON |
| `docs/development/misc.md` | Implementing convention techs (`matches_clue`, `knowledge_updates`) |
| `docs/domain/hgroup.md` | Working with H-Group conventions or terminology |

# Workflow

- Prefer minimal diffs
- Investigate before implementing
- Explain risky architectural changes first
- Do not rewrite unrelated modules
- Do not emit full files unless requested
- When a change affects something described in a doc under `docs/`, update that doc too

# Project Constraints

- TableState must remain cheap to clone
- StaticGameData should not be cloned into search nodes
- Bitfield operations are preferred in hot paths
- Convention logic must remain decoupled from core mechanics
- H-Group priority logic belongs in the H-Group layer, not generic convention infrastructure

# Search Constraints

- Avoid allocations in search hot paths
- Avoid cloning large structures during recursive evaluation
- Prefer immutable snapshots for branching
- Keep evaluator logic deterministic

## Common Commands

```bash
cargo build                              # Debug build
cargo build --release                    # Release build
cargo build --features test-support      # Build with test fixtures exposed (for benchmarks)
cargo test                               # Run all tests
cargo test <test_name>                   # Run a specific test by name (substring match)
cargo test --test '*'                    # Run integration tests only
cargo clippy                             # Lint
cargo fmt                                # Auto-format
```
