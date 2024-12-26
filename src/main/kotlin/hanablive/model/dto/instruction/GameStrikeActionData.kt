package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.model.adapter.HanabLivePlayerPOVAdapter
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : HanabLiveGameActionData(GameActionType.STRIKE) {
    override fun toStandardFormatAction(game: HanabLivePlayerPOVAdapter): GameAction {
        TODO("Not yet implemented")
    }
}
