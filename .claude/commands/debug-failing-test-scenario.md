Debug the failing integration test: $ARGUMENTS

Steps:
1. Run `cargo test $ARGUMENTS 2>&1` and capture the full output.
2. Locate the test function in `tests/` and read it.
3. If it loads a scenario, read the scenario JSON under `tests/scenarios/`.
4. Read `docs/development/testing.md` for context on scenario format and load helpers.
5. Check `git log --oneline -10` to see if a recent commit is likely responsible.
6. Identify the root cause and propose a minimal fix.
