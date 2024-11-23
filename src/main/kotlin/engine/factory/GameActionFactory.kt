package eelst.ilike.engine.factory

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction

object GameActionFactory {
    fun createClueAction(
        clueGiver: PlayerId,
        clueReceiver: PlayerId,
        clueValue: ClueValue
    ): ClueAction {
        return when(clueValue) {
            is Rank -> RankClueAction(
                    clueGiver = clueGiver,
                    clueReceiver = clueReceiver,
                    rank = clueValue,

                    )
            is Color -> ColorClueAction (
                    clueGiver = clueGiver,
                    clueReceiver = clueReceiver,
                    color = clueValue,
                )
            else -> throw IllegalArgumentException("Unrecognized clue value $clueValue")
        }
    }
}
