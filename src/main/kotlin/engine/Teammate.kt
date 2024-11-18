package eelst.ilike.engine

import eelst.ilike.engine.impl.OwnHand
import eelst.ilike.engine.impl.TeammateHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    val seatsGap: Int,
    val hand: TeammateHand,
    personalKnowledge: PersonalKnowledge,
): Player(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge
) {
    fun playsBefore(teammate: Teammate): Boolean {
        return seatsGap < teammate.seatsGap
    }

    fun getCardAtSlot(slotIndex: Int): HanabiCard {
        return hand.getSlot(slotIndex).card
    }
}
