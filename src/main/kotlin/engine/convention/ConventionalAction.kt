package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.action.GameAction

data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
) {
    companion object {
        fun <T : GameAction> from(gameAction: T, tech: ConventionTech): ConventionalAction {
            return ConventionalAction(
                action = gameAction,
                tech = tech
            )
        }
    }
}
