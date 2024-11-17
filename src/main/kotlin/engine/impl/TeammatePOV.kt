package eelst.ilike.engine.impl

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

class TeammatePOV(
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
    hand: TeammateHand,
): PlayerPOV(
    globallyAvailableInfo = globallyAvailableInfo,
    teammates = teammates,
    hand = hand,
) {
    override fun getOwnFullEmpathyCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        TODO("Not yet implemented")
    }

    override fun allCardsAreKnown(cards: Set<HanabiCard>): Boolean {
        TODO("Not yet implemented")
    }

    override fun getOwnKnownSlots(): Set<Slot> {
        TODO("Not yet implemented")
    }
}