package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.hanablive.HanabLiveGamePlayerPOV
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GamePlayActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int
) : HanabLiveGameActionData(GameActionType.PLAY) {
    override fun toStandardFormatAction(game: HanabLiveGamePlayerPOV): GameAction {
        val player = game.getPlayerMetadata(playerIndex)
        val playedSlotIndex = game.getPlayerSlot(player.playerId, order)
        return PlayAction(
            playerId = player.playerId,
            slotIndex = playedSlotIndex,
        )
    }
}
