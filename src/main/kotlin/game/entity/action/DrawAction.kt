package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot

data class DrawAction(
    val playerId: PlayerId,
    val newSlot: Slot,
): GameAction(
    actionExecutor = playerId,
    actionType = GameActionType.DRAW,
)
