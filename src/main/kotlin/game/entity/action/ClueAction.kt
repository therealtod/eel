package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

sealed class ClueAction (
    open val clueGiver: PlayerId,
    open val clueReceiver: PlayerId,
    open val value: ClueValue
): GameAction(actionExecutor = clueGiver, actionType = GameActionType.CLUE)
