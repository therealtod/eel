package eelst.ilike.engine.player

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.card.HanabiCard

class VisibleTeammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PersonalKnowledge,
    override val hand: VisibleHand,
): Teammate(
    globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
    personalKnowledge = personalKnowledge,
    hand = hand,
) {
    fun getSlots(): Set<VisibleSlot> {
        return hand.getSlots()
    }

    override fun asVisible(): VisibleTeammate {
        return this
    }

    fun getCardInSlot(slotIndex: Int): HanabiCard {
        return hand.getCardinSlot(slotIndex)
    }

    fun getVisibleHand(): VisibleHand {
        return hand
    }

    fun holdsCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return hand.getCardinSlot(slotIndex) == card
    }

    override fun isPOVProjection(): Boolean {
        return false
    }
}