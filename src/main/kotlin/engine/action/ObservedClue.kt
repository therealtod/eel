package eelst.ilike.engine.action

import eelst.ilike.game.entity.action.ClueAction

class ObservedClue(
    val clueAction: ClueAction,
    val slotsTouched: Set<Int>,
) : ObservedAction(clueAction)
