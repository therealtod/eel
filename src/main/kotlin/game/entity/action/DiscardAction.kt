package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId

class DiscardAction(
    playerId: PlayerId,
    val slotIndex: Int
) : GameAction(playerId, GameActionType.DISCARD)
