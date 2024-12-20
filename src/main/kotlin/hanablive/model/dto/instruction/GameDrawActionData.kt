package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : GameActionData(GameActionType.DRAW) {
    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        TODO("Not yet implemented")
    }
}
