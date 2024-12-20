package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : GameActionData(GameActionType.STATUS) {
    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        TODO("Not yet implemented")
    }
}
