package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : GameActionData(GameActionType.STATUS) {
    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        TODO("Not yet implemented")
    }
}
