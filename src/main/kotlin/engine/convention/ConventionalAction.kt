package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.action.GameAction

/**
 * A game action that has been generated as an instance of the given [tech]
 */
data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
)
