package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GamePlayActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int
) : GameActionData(GameActionType.PLAY) {
    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        val player = game.getPlayerInfo(playerIndex)
        val action = PlayAction(
            playerId = player.playerId,
            slotIndex = game.getPlayerSlot(player.playerId, order)
        )
        return ObservedPlay(action)
    }
}
