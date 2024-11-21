package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.clue.Clue

data class GiveClue(
    val clue: Clue,
    val to: PlayerId,
) : ExecutedAction<Clue>(gameAction = clue) {
    override fun getExecutor(): PlayerId {
        return to
    }
}
