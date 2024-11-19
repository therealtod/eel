package eelst.ilike.engine.convention

import eelst.ilike.engine.action.GameAction

data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
)
