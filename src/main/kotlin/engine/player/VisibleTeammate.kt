package eelst.ilike.engine.player

import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.card.HanabiCard

class VisibleTeammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PlayerPersonalKnowledge,
    hand: Hand,
    private val cards: List<HanabiCard>
): Teammate(
    globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
    personalKnowledge = personalKnowledge,
    hand = hand,
) {

    override fun asVisible(): VisibleTeammate {
        return this
    }

    fun getCardInSlot(slotIndex: Int): HanabiCard {
        return cards[slotIndex - 1]
    }

    fun holdsCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return getCardInSlot(slotIndex) == card
    }

    override fun isPOVProjection(): Boolean {
        return false
    }
}