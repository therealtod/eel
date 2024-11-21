package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.clue.GameAction

abstract class ExecutedAction<T: GameAction>(private val gameAction: T): PlayerAction<T> {
    override fun getAction(): T {
        return gameAction
    }
    abstract fun getExecutor(): PlayerId
}
