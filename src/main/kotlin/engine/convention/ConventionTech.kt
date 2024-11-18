package eelst.ilike.engine.convention

import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.action.GameAction


interface ConventionTech {
    val name: String
    fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun getGeneratedKnowledge(action: GameAction): GeneratedKnowledge
}
