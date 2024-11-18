package eelst.ilike.engine.convention

import eelst.ilike.game.action.GameAction

data class ConventionalAction(
    val action: GameAction,
    val tech: ConventionTech,
) {
    fun getGeneratedKnowledge(): GeneratedKnowledge {
        return tech.getGeneratedKnowledge() // TODO() questo wrapper ConventionalAction impedisce di risolvere la action per tipo
    }
}
