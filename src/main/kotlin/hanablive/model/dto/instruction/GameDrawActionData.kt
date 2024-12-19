package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : GameActionData(GameActionType.DRAW) {
    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        TODO("Not yet implemented")
    }
}
