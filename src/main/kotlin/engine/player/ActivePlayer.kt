package eelst.ilike.engine.player

import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.PlayerId

class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    hand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
    teammates: Set<Teammate>,
): Player(
    playerId = playerId,
    playerIndex = playerIndex,
    hand = hand,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge,
) {
    val playerPOV: PlayerPOV = ActivePlayerPOV(
        globallyAvailableInfo = globallyAvailableInfo,
        teammates = teammates,
        ownHand = ownHand
    )

    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }
}