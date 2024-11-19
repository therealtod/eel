package eelst.ilike.engine.convention

import eelst.ilike.engine.player.PlayerPOV


interface ConventionTech {
    val name: String
    // fun recognize(playerPOV: ActivePlayerPOV): Set<GameAction>
    fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction>
    fun overrides(otherTech: ConventionTech): Boolean
}
