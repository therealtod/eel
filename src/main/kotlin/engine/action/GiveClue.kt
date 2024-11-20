package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.action.Clue

data class GiveClue(
    val clue: Clue,
    override val from: PlayerId,
    val to: PlayerId,
) : GameAction(from)
