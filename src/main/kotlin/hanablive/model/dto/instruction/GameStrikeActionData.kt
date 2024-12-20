package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : GameActionData(GameActionType.STRIKE) {
    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        TODO("Not yet implemented")
    }
}
