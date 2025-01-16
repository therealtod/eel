package eelst.ilike.game.entity.action


import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.slot.Slot

data class PlayAction(
    val playerId: PlayerId,
    val slotIndex: Int,
) : GameAction(
    actionExecutor = playerId,
    actionType = GameActionType.PLAY,
)
