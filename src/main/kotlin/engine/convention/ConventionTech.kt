package eelst.ilike.engine.convention

import eelst.ilike.engine.player.ActivePlayerPOV


interface ConventionTech {
    val name: String
    fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction>
    fun overrides(otherTech: ConventionTech): Boolean
}
