package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId

sealed class GameAction(val actionExecutor: PlayerId, val actionType: GameActionType)
