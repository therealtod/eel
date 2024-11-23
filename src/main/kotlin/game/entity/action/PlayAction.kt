package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId

class PlayAction(
    playerId: PlayerId,
    val slotIndex: Int,
) : GameAction(
    actionExecutor = playerId,
    actionType = GameActionType.PLAY,
)
