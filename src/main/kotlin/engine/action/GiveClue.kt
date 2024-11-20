package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.action.Clue

data class GiveClue(
    val clue: Clue,
    val to: PlayerId,
) : GameAction()
