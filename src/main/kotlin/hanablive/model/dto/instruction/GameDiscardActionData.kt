package eelst.ilike.hanablive.model.dto.instruction


import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDiscardActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
    val failed: Boolean,
) : GameActionData(GameActionType.DISCARD) {
    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        val player = game.getPlayerMetadata(playerIndex)
        val discardedSlotIndex = game.getPlayerSlot(player.playerId, order)
        return DiscardAction(
            playerId = player.playerId,
            slotIndex = discardedSlotIndex
        )
    }
}
