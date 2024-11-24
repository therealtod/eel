package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId

data class DiscardAction(
    val playerId: PlayerId,
    val slotIndex: Int
) : GameAction(playerId, GameActionType.DISCARD)
