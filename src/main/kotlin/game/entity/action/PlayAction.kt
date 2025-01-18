package eelst.ilike.game.entity.action


import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot

data class PlayAction(
    val playerMetadata: PlayerMetadata,
    val slotIndex: Int,
) : GameAction(
    actionExecutor = playerMetadata,
    actionType = GameActionType.PLAY,
)
