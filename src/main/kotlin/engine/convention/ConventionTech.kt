package eelst.ilike.engine.convention

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.player.PlayerPOV


interface ConventionTech {
    val name: String
    fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(observedAction: ObservedAction, playerPOV: PlayerPOV): Boolean {
        TODO()
    }
}
