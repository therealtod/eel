
## H-Group Convention Terminology

| Term           | Definition                                                                                                                                                                                                        |
|----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chop**       | The oldest unclued card in a player's hand — the card most at risk of being discarded. Implemented by `get_chop_index` in `convention/hgroup/h_group_core.rs`.                                                    |
| **Clue focus** | The card a clue is "about". If the clue touches the chop, the chop is the focus. Otherwise, the focus is the leftmost (newest, slot 1) card that was not previously clued. Implemented by `get_clue_focus_index`. |
| **Slot**       | Slot 1 = newest card (most recently drawn). Slot N = oldest card (chop when unclued).                                                                                                                             |
| **Early Game** | The opening phase, lasting until the first time any player performs a `DiscardChop` (discards their chop). Clues, plays, and even known-trash discards keep the game in the Early Game; only a chop discard ends it. Implemented by `is_early_game` in `convention/hgroup/h_group_core.rs` (also exposed as `KnowledgeAwareGameState::is_early_game`). |
| **Blind-play** | Playing a card not touched by any clue (and without narrowing empathy), relying on implicit convention-driven information.                                                                                        |

Scenario docstrings and tests refer to players by name: **Alice** = player 0 (always on turn in scenarios), **Bob** = 1,
**Cathy** = 2, **Donald** = 3, **Emily** = 4.

## H-Group Principles

The engine encodes the foundational H-Group principles in `convention/hgroup/h_group_core.rs`:

| Principle | Encoded as | Meaning |
|-----------|-----------|---------|
| **MCVP** — Minimum Clue Value Principle | `is_minimal_clue_value_compliant` | A clue must either touch a not-yet-gotten card or cause more than one card to become "going to play". |
| **GTP** — Good Touch Principle | `is_good_touch_principle_compliant` / `count_bad_touches` | A clue should not touch a card that is trash or a duplicate of an already-saved identity (a *bad touch*). |
| **Save Principle** | `is_save_principle_protected` / `is_forced_save` | The team must save from discard every card it cannot afford to lose. |
| **Early Game restriction** | `GameActionFilter::early_game_chop_discard` (applied in `candidate_actions_with_provenance`) | While the game is still in the [Early Game](#h-group-convention-terminology), a player may not discard their chop if they have a productive, good-touch-compliant tech available (a play, a known-trash discard, or a GTP-compliant clue — never a stall fallback). A *known-trash* chop discard is always allowed (it is a `DiscardKnownTrash`, not a `DiscardChop`). |

The Save Principle's **protected set** has three parts:

1. **Playable cards** — any identity a play clue tech could focus (`is_play_clue_focusable`:
   immediately playable, or focusable via a delayed play / prompt / finesse, approximated by "every
   connecting card down to the stack is present in some hand").
2. **Critical cards** — the last live copy of a still-needed identity.
3. **A variant-dependent set** (`Variant::always_save_cards`; the 2s in the base game) — saved even
   before they become critical. (5s are single-copy, hence always critical.)

**Precedence: Save Principle > GTP / tempo.** The evaluator softens two penalties with positive
*relief* terms (subtracted from the base penalty, floored at 0, kept non-zero) when the Save
Principle is at stake:

- **Forced-save bad touch** (`forced_save_bad_touch_relief`): saving a protected card on chop
  sometimes cannot avoid a bad touch (e.g. a colour critical-save that also touches a dead card of
  that suit). Because the save is *forced* by the higher-priority Save Principle, each such bad
  touch is charged `bad_touch_penalty - forced_save_bad_touch_relief` instead of the full rate —
  small enough that the engine never lets a protected card be discarded to avoid the collateral
  touch, but non-zero so a cleaner save of the same card is still preferred.
- **Known-playable discard under save pressure** (`save_pressure_known_playable_relief`): the
  `discard_while_known_playable_penalty` is softened when `save_principle_card_in_danger` holds
  (another player's unclued chop holds a protected card they would be forced to discard), excusing a
  player who spends the turn on the save instead of cashing their own known play. Note this only
  bites when the deferral is realised as a *discard*; a save delivered as a clue carries no such
  penalty to relieve.