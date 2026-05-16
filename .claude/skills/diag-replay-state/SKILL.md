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
2. **Write a throwaway `tests/_diag.rs`** that steps the replay to that turn and prints:
   - Active player, hand contents, `playing_stacks`.
   - Per own-hand card: `combined_possible_identities` bits, `inferred_identities` bits,
     `is_touched`, signals.
   - `playable_cards` mask.
   - Hypotheses on the player's knowledge (tier, cohort, immediate updates).
   - Each tech's `game_actions(&pov)` output — tells you *who* is proposing the bad move.
3. **Run** `cargo test --test _diag diag_<name> -- --nocapture`.
4. **Decode bits** (suit-major: ids 0–4 = suit 0 ranks 1–5, ids 5–9 = suit 1, …) and find
   the layer that disagrees with what you expected.
5. **Walk backwards through the action log** to find the event that introduced the bad
   state. The hanablive action list (`type`/`target`/`value`) is your timeline.
6. **Fix, re-run the original test, then `rm tests/_diag.rs`** — never check it in.

## Diagnostic template

Adjust the convention set list to match what the failing test uses. The `_` prefix on the
filename keeps it visually distinct from real tests.

```rust
// tests/_diag.rs
use eel::engine::convention::convention_set::ConventionSet;
use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::convention::hgroup::tech::{
    blind_play::BlindPlay,
    critical_save::{ColorCriticalSave, RankCriticalSave},
    delayed_play_clue::DelayedPlayClue,
    direct_play_clue::DirectPlayClue,
    discard_chop::DiscardChop,
    five_save::FiveSave,
    play_known_playable::PlayKnownPlayable,
    simple_finesse::SimpleFinesse,
    simple_prompt::SimplePrompt,
    two_save::TwoSave,
};
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
    let techs: Vec<Box<dyn ConventionTech>> = vec![
        Box::new(PlayKnownPlayable),
        Box::new(BlindPlay),
        Box::new(DirectPlayClue),
        Box::new(DelayedPlayClue),
        Box::new(SimplePrompt),
        Box::new(SimpleFinesse),
        Box::new(ColorCriticalSave),
        Box::new(RankCriticalSave),
        Box::new(FiveSave),
        Box::new(TwoSave),
        Box::new(DiscardChop),
    ];
    let conv = HGroupConventionSet::new(techs);
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
            "  tier={} cohort={} immediate={:?} trigger?={}",
            h.tier,
            h.cohort_id,
            h.immediate,
            h.trigger.is_some()
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
| Tech proposes action but engine picks different action | Search ranking issue — also dump `strategy.scored_actions(&pov, &conv)` to compare totals. |

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

The diagnostic file is not a test of the system; it's a probe. Always `rm tests/_diag.rs`
before committing. If the bug surfaces a missing assertion that's worth keeping, add a
real `#[test]` to `tests/replay_regression.rs` (or the appropriate file) and delete the
diag.
