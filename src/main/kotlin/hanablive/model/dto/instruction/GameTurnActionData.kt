package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGamePlayerPOV
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : HanabLiveGameActionData(GameActionType.TURN) {
    override fun toStandardFormatAction(game: HanabLiveGamePlayerPOV): GameAction {
        TODO("Not yet implemented")
    }
}