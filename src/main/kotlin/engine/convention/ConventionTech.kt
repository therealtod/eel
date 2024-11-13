package eelst.ilike.engine.convention

import eelst.ilike.engine.PlayerPOV


interface ConventionTech {
    val name: String
    fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction>
}
