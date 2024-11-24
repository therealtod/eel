package eelst.ilike.engine.convention

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech {
    val name: String
    fun getGameActions(playerPOV: PlayerPOV): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
}
