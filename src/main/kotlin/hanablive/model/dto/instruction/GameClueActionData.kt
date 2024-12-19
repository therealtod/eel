package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.game.Game
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameClueActionData(
    val clue: Clue,
    val giver: Int,
    val list: List<Int>,
    val target: Int,
    val turn: Int,
) : GameActionData(GameActionType.CLUE) {
    data class Clue(
        val type: Int,
        val value: Int,
    )
}
