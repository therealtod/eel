package eelst.ilike.engine.impl

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayerPOV(
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
    val ownHand: OwnHand,
): PlayerPOV(
    globallyAvailableInfo = globallyAvailableInfo,
    teammates = teammates,
    hand = ownHand,
) {
    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        val allKnownCards = teammates
            .flatMap { it.getKnownCards() } +
                getOwnKnownCards()
        return allKnownCards.containsAll(cards)
    }

    override fun getOwnKnownSlots(): Set<KnownSlot> {
        return ownHand.getKnownSlots(getVisibleCards())
    }

    override fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    override fun getOwnFullEmpathyCards(): List<HanabiCard> {
        return ownHand
            .map {
                it.getEmpathy(getVisibleCards(), globallyAvailableInfo.suites)
            }.filter {
                it.size == 1
            }.flatten()
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.card) }.toSet()
    }
}