package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.model.adapter.HanabLivePlayerPOVAdapter
import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : HanabLiveGameActionData(GameActionType.DRAW) {
    override fun toStandardFormatAction(game: HanabLivePlayerPOVAdapter): GameAction {
        TODO("Not yet implemented")
    }
}
