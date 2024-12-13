package eelst.ilike.engine.player

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.card.HanabiCard

class VisibleTeammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PlayerPersonalKnowledge,
    private val visibleHand: VisibleHand
): Teammate(
    globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
    hand = visibleHand,
) {

    override fun asVisible(): VisibleTeammate {
        return this
    }

    fun getCardInSlot(slotIndex: Int): HanabiCard {
        return visibleHand.getCardInSlot(slotIndex)
    }

    fun holdsCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return getCardInSlot(slotIndex) == card
    }

    override fun isPOVProjection(): Boolean {
        return false
    }
}