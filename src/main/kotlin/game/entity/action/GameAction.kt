package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

sealed class GameAction(val actionExecutor: PlayerMetadata, val actionType: GameActionType)
