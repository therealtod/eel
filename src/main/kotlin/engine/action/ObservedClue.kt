package eelst.ilike.engine.action

import eelst.ilike.game.entity.action.ClueAction

class ObservedClue(
    clueAction: ClueAction,
    val slotsTouched: Set<Int>,
) : ObservedAction<ClueAction>(clueAction)
