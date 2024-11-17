package eelst.ilike.engine

import eelst.ilike.engine.impl.TeammateHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.VisibleSlot
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    val playerId: PlayerId,
    val seatsGap: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val hand: TeammateHand,
    val personalInfo: TeammatePersonalInfo,
) {
    val visibleCards: List<HanabiCard> = TODO()

    fun playsBefore(teammate: Teammate): Boolean {
        return seatsGap < teammate.seatsGap
    }

    fun getKnownCards(): List<HanabiCard> {
        TODO()
        // return playerPOV.hand.filter { it.isKnown(playerPOV) }.map { it.getCardIdentity(playerPOV) }
    }

    fun knows(slotIndex: Int): Boolean {
        return hand.getSlot(slotIndex)
            .fromOwnerPOV(personalSlotInfo = personalInfo.getSlotInfo(slotIndex))
            .isKnown()
    }

    fun getSlot(slotIndex: Int): VisibleSlot{
        return hand.getSlot(slotIndex)
    }
}