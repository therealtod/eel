package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId

abstract class Player(
    val playerId: PlayerId,
    val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val hand: InterpretedHand,
    val playerPOV: PlayerPOV,
) {
    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }
}
