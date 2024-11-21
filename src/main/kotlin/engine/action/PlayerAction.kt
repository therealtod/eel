package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.clue.GameAction

interface PlayerAction<T: GameAction> {
    fun getAction(): T
}
