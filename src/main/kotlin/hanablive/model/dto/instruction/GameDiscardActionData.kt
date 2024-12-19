package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDiscardActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
    val failed: Boolean,
) : GameActionData(GameActionType.DISCARD) {
    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        val player = game.getPlayer(playerIndex)
        val action = DiscardAction(
            playerId = player.playerId,
            slotIndex = game.getPlayerSlot(player.playerId, order)
        )
        return ObservedDiscard(action)
    }
}
