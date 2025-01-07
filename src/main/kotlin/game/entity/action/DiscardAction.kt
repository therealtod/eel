package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.player.PlayerId


data class DiscardAction(
    val playerId: PlayerId,
    val slotIndex: Int
) : GameAction(playerId, GameActionType.DISCARD)
