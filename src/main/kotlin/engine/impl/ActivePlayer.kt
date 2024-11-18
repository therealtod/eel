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
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
    teammates: Set<Teammate>,
): Player(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge,
) {
    val playerPOV: PlayerPOV

    init {
        playerPOV = ActivePlayerPOV(
            globallyAvailableInfo = globallyAvailableInfo,
            teammates = teammates,
            ownHand = ownHand
        )
    }

    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }
}