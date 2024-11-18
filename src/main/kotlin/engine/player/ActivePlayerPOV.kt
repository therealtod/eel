package eelst.ilike.engine.player

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayerPOV(
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
    val ownHand: OwnHand,
) : PlayerPOV(
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
        return ownHand.getKnownSlots()
    }

    override fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.card) }.toSet()
    }
}