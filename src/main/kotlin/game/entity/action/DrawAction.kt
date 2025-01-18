package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

data class DrawAction(
    val playerMetadata: PlayerMetadata,
) : GameAction(
    playerMetadata,
    actionType = GameActionType.DRAW,
)
