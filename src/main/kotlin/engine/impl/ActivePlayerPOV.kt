package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayerPOV(
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
    hand: InterpretedHand,
): PlayerPOV(
    globallyAvailableInfo = globallyAvailableInfo,
    teammates = teammates,
    hand = hand,
) {
    override fun allCardsAreKnown(cards: Set<HanabiCard>): Boolean {
        TODO("Not yet implemented")
    }

    override fun getOwnKnownSlots(): Set<Slot> {
        return hand.getKnownSlots(this)
    }

    override fun getOwnFullEmpathyCards(): List<HanabiCard> {
        return hand.filter { it.getEmpathy(this).size == 1 }.map { it.getCard() }
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.getCard()) }.toSet()
    }
}