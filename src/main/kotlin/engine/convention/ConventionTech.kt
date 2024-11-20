package eelst.ilike.engine.convention

import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.player.PlayerPOV


interface ConventionTech {
    val name: String
    fun getGameActions(playerPOV: PlayerPOV): Set<GameAction>
    fun getConventionalActions(playerPOV: PlayerPOV): Set<ConventionalAction>
    fun overrides(otherTech: ConventionTech): Boolean
}
