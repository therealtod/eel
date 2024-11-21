package eelst.ilike.engine.convention

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.game.entity.clue.GameAction

data class ConventionalAction<T: GameAction>(
    val action: PlayerAction<T>,
    val tech: ConventionTech<in T>,
)
