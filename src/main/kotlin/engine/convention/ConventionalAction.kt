package eelst.ilike.engine.convention

import eelst.ilike.game.action.GameAction

data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
)
