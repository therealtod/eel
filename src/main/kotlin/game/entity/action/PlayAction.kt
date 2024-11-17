package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId

data class PlayAction(
    val playerId: PlayerId,
    val slotIndex: Int,
) : GameAction(
    actionExecutor = playerId,
    actionType = GameActionType.PLAY,
)
