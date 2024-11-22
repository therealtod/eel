package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.action.ClueAction

data class GiveClue(
    val clue: ClueAction,
    val to: PlayerId,
) : GameAction()
