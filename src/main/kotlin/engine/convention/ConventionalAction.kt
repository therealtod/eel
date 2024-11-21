package eelst.ilike.engine.convention

import eelst.ilike.engine.action.PlayerAction

data class ConventionalAction(
    val action: PlayerAction,
    val tech: ConventionTech,
)
