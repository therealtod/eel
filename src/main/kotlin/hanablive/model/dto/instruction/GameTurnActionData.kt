package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : GameActionData(GameActionType.TURN) {
    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        TODO("Not yet implemented")
    }
}