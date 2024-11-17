package eelst.ilike.engine.impl

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.Player
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.PlayerId

class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    hand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    val playerPOV: PlayerPOV,
): Player(
    playerId = playerId,
    playerIndex = playerIndex,
    hand = hand,
    globallyAvailableInfo = globallyAvailableInfo
) {
    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }
}