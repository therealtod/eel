package eelst.ilike.engine.convention

import eelst.ilike.game.entity.action.GameAction

data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
)
