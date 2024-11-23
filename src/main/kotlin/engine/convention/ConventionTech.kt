package eelst.ilike.engine.convention

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech <T: GameAction> {
    val name: String
    fun getGameActions(playerPOV: PlayerPOV): Set<T>
    fun overrides(otherTech: ConventionTech<T>): Boolean
}
