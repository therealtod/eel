package eelst.ilike.engine.player

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    override val playerPOV: PlayerPOV,
): BasePlayer(
    playerId = playerId,
    playerIndex = playerIndex,
) {

    fun getLegalActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    override fun getCardAtSlot(slotIndex: Int): HanabiCard {
        throw UnsupportedOperationException()
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }

    override fun buildPlayerPOV(
        globallyAvailableInfo: GloballyAvailableInfo,
        ownHand: OwnHand,
        players: Set<BasePlayer>
    ): PlayerPOV {
        return PlayerFactory.createPlayerPOV(
            globallyAvailableInfo = globallyAvailableInfo,
            ownHand = ownHand,
            personalKnowledge = TODO()
        )
    }
}