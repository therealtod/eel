package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.RankClueAction
import eelst.ilike.hanablive.HanabLiveGame
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

    override fun toStandardFormatAction(game: HanabLiveGame): GameAction {
        val clueValue = game.getClueValue(clue)
        val clueGiver = game.getPlayerMetadata(giver)
        val clueReceiver = game.getPlayerMetadata(target)

        val action = when(clueValue) {
            is Color -> ColorClueAction(
                clueGiver = clueGiver.playerId,
                clueReceiver = clueReceiver.playerId,
                color = clueValue,
            )
            is Rank -> RankClueAction(
                clueGiver = clueGiver.playerId,
                clueReceiver = clueReceiver.playerId,
                rank = clueValue,
            )
            else -> throw UnsupportedOperationException("Unsupported clue value: $clueValue")
        }
        return action
    }
}
