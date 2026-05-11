
## H-Group Convention Terminology

| Term           | Definition                                                                                                                                                                                                        |
|----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chop**       | The oldest unclued card in a player's hand — the card most at risk of being discarded. Implemented by `get_chop_index` in `convention/hgroup/h_group_core.rs`.                                                    |
| **Clue focus** | The card a clue is "about". If the clue touches the chop, the chop is the focus. Otherwise, the focus is the leftmost (newest, slot 1) card that was not previously clued. Implemented by `get_clue_focus_index`. |
| **Slot**       | Slot 1 = newest card (most recently drawn). Slot N = oldest card (chop when unclued).                                                                                                                             |
| **Blind-play** | Playing a card not touched by any clue (and without narrowing empathy), relying on implicit convention-driven information.                                                                                        |

Scenario docstrings and tests refer to players by name: **Alice** = player 0 (always on turn in scenarios), **Bob** = 1,
**Cathy** = 2, **Donald** = 3, **Emily** = 4.