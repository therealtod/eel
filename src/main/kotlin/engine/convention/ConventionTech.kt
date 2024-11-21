package eelst.ilike.engine.convention

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.clue.GameAction


interface ConventionTech<T: GameAction> {
    val name: String
    fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction<T>>
    fun overrides(otherTech: ConventionTech<*>): Boolean
    fun matches(observedAction: ObservedAction<T>, playerPOV: PlayerPOV): Boolean {
        TODO()
    }
}
