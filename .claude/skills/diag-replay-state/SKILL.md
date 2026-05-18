---
name: diag-replay-state
description: |
  Debug a failing replay regression test by dumping the engine's internal state at the
  failing turn — per-card empathy, signals, hypotheses, and what each convention tech
  proposes. Use when an integration test in tests/replay_regression.rs fails and you
  need to understand WHY the engine chose its action (rather than just that it did).
  Faster and cleaner than println debugging or pure code reasoning.
---

# Diagnose a replay regression failure

When a `tests/replay_regression.rs` (or similar replay-driven) test fails, the assertion
only tells you *what* the engine did wrong. The decision is the output of a long chain:
clue interpretation → empathy update → convention tech firing → tree search ranking.
Find the divergence by **dumping engine state at the failing turn** with a throwaway
diagnostic test, then walking backwards to the corrupted layer.

## The pattern

1. **Read the failing test** to identify the replay file and turn number (e.g.
   `engine_action_at_turn("plays_known_trash.json", 18)`).
2. **Run the script** — it generates `tests/_diag.rs` and executes it in one step:
   ```
   scripts/diag.sh <replay_file.json> <turn> [optional_name]
   ```
   For example: `scripts/diag.sh plays_known_trash.json 18 trash_play`
3. **Decode bits** (suit-major: ids 0–4 = suit 0 ranks 1–5, ids 5–9 = suit 1, …) and find
   the layer that disagrees with what you expected.
4. **Walk backwards through the action log** to find the event that introduced the bad
   state. The hanablive action list (`type`/`target`/`value`) is your timeline.
5. **Fix and re-run the original test.** `tests/_diag.rs` is gitignored — leave it or
   delete it; it will never be committed.

`scripts/diag.sh` uses `HGroupConventionSet::default()` which always reflects the current
full tech list. If you need a custom tech subset, edit the generated `tests/_diag.rs`
directly and re-run with `cargo test --test _diag diag_<name> -- --nocapture`.

## Diagnostic template (for reference / custom subsets)

```rust
// tests/_diag.rs  (gitignored — safe to leave in place)
use eel::engine::convention::convention_set::ConventionSet;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::knowledge::player_pov::PlayerPOV;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::external::hanablive::Game;

#[test]
fn diag_<short_name>() {
    let path = format!(
        "{}/tests/replays/<REPLAY_FILE>.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&path).unwrap();
    let game = Game::from_json(&json).unwrap();
    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv).unwrap();
    runner.step_to_turn(<TURN>).unwrap();

    let strategy = TreeActionSelectionStrategy::default();
    println!("recommendation: {:?}", runner.engine_recommendation(&strategy));

    let pov = runner
        .game
        .player_pov(runner.game.table_state.active_player_index);
    println!("active player: {}", pov.active_player_index());
    println!("stacks: {:?}", pov.table_state().playing_stacks);

    let knowledge = pov.team_knowledge().player(pov.active_player_index());
    let mut hand = knowledge.own_hand;
    while hand != 0 {
        let idx = hand.trailing_zeros() as u8;
        let empathy = knowledge.combined_possible_identities(
            idx,
            pov.table_state(),
            &pov.static_data().variant,
        );
        let inferred = knowledge.possible_identities(idx);
        println!(
            "  deck[{}]: empathy={:025b} inferred={:?} touched={} signals={:?}",
            idx,
            empathy.as_bits(),
            inferred.map(|m| format!("{:025b}", m.as_bits())),
            pov.is_touched(idx),
            &knowledge.signals[idx as usize],
        );
        hand &= !(1u64 << idx);
    }
    println!(
        "playable_cards: {:025b}",
        pov.table_state().playable_cards(pov.static_data())
    );

    println!("hypotheses ({}):", knowledge.hypotheses.len());
    for h in &knowledge.hypotheses {
        println!(
            "  id={} tier={} cohort={} alt_group={:?} immediate={:?} trigger={:?}",
            h.id,
            h.tier,
            h.cohort_id,
            h.alt_group,
            h.immediate,
            h.trigger
        );
    }

    for tech in conv.techs() {
        let actions = tech.game_actions(&pov);
        if !actions.is_empty() {
            println!("tech {} proposes: {:?}", tech.name(), actions);
        }
    }
}
```

## Reading the output

- **Bit decoding (5-suit No Variant).** Id = `suit * 5 + (rank - 1)`. Suits in hanablive
  order: 0=R, 1=Y, 2=G, 3=B, 4=P. So bit 0=R1, bit 5=Y1, bit 10=G1, bit 15=B1, bit 20=P1.
  Print with `{:025b}` for a fixed-width 25-bit mask, then count from the right.
- **`empathy` vs `inferred`.** `combined_possible_identities` for an own-unseen card returns
  `effective_inferred_mask` only (baseline ∩ tier-0 hypothesis cohorts). If they disagree,
  a hypothesis is active. If `empathy` is *wider* than you'd expect, a clue narrowing
  failed silently (likely an empty-intersection `narrow_inferred` no-op).
- **`tech ... proposes`.** Pinpoints the tech responsible. `PlayKnownPlayable` proposing a
  bad play means empathy is wrong (start there). `BlindPlay` proposing it means a
  `Signal::Play` is attached incorrectly. Clue tech proposing a bad clue means the focus
  calculation or the playability check is off.

## Common failure shapes

| Symptom in dump | Likely cause |
|---|---|
| Empathy includes ids that the clue history rules out | Raw clue mask narrowing skipped because intersection went empty in `narrow_inferred` (silent drop). |
| Empathy is a singleton but wrong identity | Hypothesis cohort baked into baseline using too-narrow tech mask. |
| `Signal::Play` on a touched card that's now trash | Signal lifetime / deadline not respecting played-stack progress. |
| Tech proposes action but engine picks different action | Search ranking issue — see "Dumping the search ranking" below. |
| Two hypotheses on same cohort, tier=0 narrows to wrong id, tier=1 to right id | A higher-priority tech is `matches_clue`-matching when it shouldn't. Tighten that tech's `matches_clue`, not the lower-priority one. |
| Search rollout shows a card being played that is actually trash, with no strike | Phantom-play hallucination: empathy is ambiguous-but-all-playable, truth disagrees, but `apply_play` falls into the phantom branch without consulting truth. Fix in `apply_play`, not in the proposing tech. |

## Dumping the search ranking

When the tech that proposed the bad move looks fine in isolation but the search still
picks something else (or the inverse — the search picks a bad action that no tech "owns"),
dump the per-candidate scored line. The PV shows what the search believes will happen
turn-by-turn, which is usually where the bug lives.

```rust
let strategy = TreeActionSelectionStrategy::default();
let nodes = strategy.scored_actions(&pov, &conv);
for n in nodes.iter().take(6) {
    println!("  total={:.3} action={:?}", n.total_score, n.action);
    for (i, step) in n.line.iter().enumerate() {
        println!(
            "    [{i}] imm={:+.2} {} {:?}",
            step.immediate_bonus, step.tech_name, step.action
        );
    }
}
```

Read the PVs for the actual choice and its losing rivals side-by-side: an unexpected
`PlayKnownPlayable` of a trash card in the PV, or two lines converging on the same total
score (a tie broken by enumeration order), tells you exactly which layer to fix.

## Dumping a sub-search

When the suspect decision is mid-rollout (not at the root), construct the intermediate
state by applying the prefix actions yourself, then call `scored_actions` on the resulting
POV. This works for both replay-loaded and scenario-loaded states.

```rust
// e.g. "after Alice's discard, what does Bob see?"
let alice_knowledge = state.team_knowledge.player(0).clone();
let alice_team = state.team_knowledge.clone();
let alice_table = state.table_state().clone();
let alice_pov = LightweightPlayerPOV::new(
    0, &alice_knowledge, &alice_team, &alice_table, &static_data,
);
state.apply(&alice_discard, &conventions, &alice_pov);
state.advance_turn();
// now build Bob's POV and call scored_actions on it
```

The clone-then-borrow gymnastics are needed because `apply` takes `&mut self.state`
while the truth POV holds `&` borrows of the same fields.

## Scenario tests, not just replays

The same dump pattern works for `tests/search_regression.rs` (scenario-driven) tests —
substitute `common::load_scenario_by_name_with_knowledge("search/<name>")` for the
`ReplayRunner::from_hanablive` setup. The rest of the diagnostic (empathy dump, tech
proposals, scored_actions) is identical.

## Bisecting recent code changes

If a test was passing before a session's edits and now fails, bisect with `git stash`:
stash a single file's worth of changes, re-run the test, restore. Iterate until you
find the change that broke it. Useful when an architectural change (e.g. threading a
new parameter through `apply`) interacts non-obviously with downstream tests.

## When to use this skill vs. just reading code

Use the diagnostic dump when:
- The state has been mutated over many turns (clues, plays, hypotheses) and you can't
  reliably reconstruct it by hand.
- You suspect a specific layer (empathy, signals, tech) but want to confirm before fixing.
- A test passes on simple scenarios but fails on a replay — the dump tells you what
  differs between the two.

Skip it when:
- The failing assertion is one or two layers from the bug (e.g. a unit test on a single
  tech). Read the tech and the test instead.
- You're changing a public API and the failures are mechanical — fix them directly.

## Cleanup

`tests/_diag.rs` is gitignored — it can never be accidentally committed, so no cleanup is
required. If the bug surfaces a missing assertion that's worth keeping, add a real `#[test]`
to `tests/replay_regression.rs` (or the appropriate file).
