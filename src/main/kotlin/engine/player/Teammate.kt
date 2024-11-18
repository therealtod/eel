package eelst.ilike.engine.player

import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    val seatsGap: Int,
    override val hand: TeammateHand,
    personalKnowledge: PersonalKnowledge,
) : Player(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge,
    hand = hand,
) {
    fun playsBefore(teammate: Teammate): Boolean {
        return seatsGap < teammate.seatsGap
    }

    fun getCardAtSlot(slotIndex: Int): HanabiCard {
        return hand.getSlot(slotIndex).card
    }

    fun getSlot(slotIndex: Int): VisibleSlot {
        return hand.getSlot(slotIndex)
    }

    fun getSlotFromTeammatePerspective(slotIndex: Int): InterpretedSlot {
        return hand.getSlot(slotIndex)
    }
}
