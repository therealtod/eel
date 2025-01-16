package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.player.PlayerId

data class DrawAction(
    val playerId: PlayerId,
) : GameAction(
    actionExecutor = playerId,
    actionType = GameActionType.DRAW,
)
