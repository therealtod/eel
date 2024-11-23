package eelst.ilike.engine.convention

import eelst.ilike.game.entity.action.GameAction

data class ConventionalAction<T: GameAction>(
    val action: T,
    val tech: ConventionTech<T>,
)
