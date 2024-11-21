package eelst.ilike.engine.action

import eelst.ilike.game.entity.clue.GameAction

interface PlayerAction {
    fun getAction(): GameAction
}
