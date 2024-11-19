package eelst.ilike.engine.player

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    playerPOV: PlayerPOV,
): BasePlayer(
    playerId = playerId,
    playerIndex = playerIndex,
    playerPOV = playerPOV,
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

    override fun getSlots(): Set<InterpretedSlot> {
        TODO("Not yet implemented")
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        val slot = ownHand.getSlot(slotIndex)
        return slot.hasKnownIdentity(card)
    }
}