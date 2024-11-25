package eelst.ilike.engine.convention

import eelst.ilike.game.entity.action.GameAction

data class ConventionalAction<T: GameAction>(
    val action: GameAction,
    val tech: ConventionTech<T>,
) {
    companion object {
        fun <T: GameAction> from (gameAction: T, tech: ConventionTech<T>): ConventionalAction<T> {
            return ConventionalAction(
                action = gameAction,
                tech = tech
            )
        }
    }
}
