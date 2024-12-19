package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : GameActionData(GameActionType.STRIKE) {
    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        TODO("Not yet implemented")
    }
}
