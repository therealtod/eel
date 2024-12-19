package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
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

    override fun toObservedAction(game: HanabLiveGame): ObservedAction {
        val clueValue = game.getClueValue(clue)
        val clueGiver = game.getPlayerInfo(giver)
        val clueReceiver = game.getPlayerInfo(target)

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
        return ObservedClue(
            clueAction = action,
            slotsTouched = game.getPlayerSlots(clueReceiver.playerId, list)
        )
    }
}
