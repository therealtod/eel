package eelst.ilike.engine.player

import eelst.ilike.engine.EngineCommon
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayerPOV(
    playerId: PlayerId,
    playerIndex: Int,
    hand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
) : PlayerPOV(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    teammates = teammates,
    hand = hand
) {
    override fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(this) }
        return EngineCommon.getPrunedAction(candidateActions)
    }

    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        val allKnownCards = teammates
            .flatMap { it.getKnownCards() } +
                getOwnKnownCards()
        return allKnownCards.containsAll(cards)
    }

    override fun getOwnKnownSlots(): Set<KnownSlot> {
        return hand.getKnownSlots()
    }

    override fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.card) }.toSet()
    }
}