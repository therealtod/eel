package eelst.ilike.engine.player

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
    val hand: VisibleHand,
    val seatsGap: Int,
) : ConventionsUsingPlayer(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge,
) {
    fun playsBefore(otherTeammate: Teammate): Boolean {
        return seatsGap < otherTeammate.seatsGap
    }

    fun getSlotFromTeammatePOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    fun getCardAtSlot(slotIndex: Int): HanabiCard {
        return hand.getSlot(slotIndex).card
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return hand.getSlot(slotIndex).card == card
    }
}
