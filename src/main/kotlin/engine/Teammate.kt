package eelst.ilike.engine

import eelst.ilike.engine.impl.TeammateHand
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

data class Teammate(
    val playerId: PlayerId,
    val seatsGap: Int,
    val playerPOV: PlayerPOV,
    val hand: TeammateHand
) {
    fun playsBefore(teammate: Teammate): Boolean {
        return seatsGap < teammate.seatsGap
    }

    fun getKnownCards(): List<HanabiCard> {
        TODO()
        // return playerPOV.hand.filter { it.isKnown(playerPOV) }.map { it.getCardIdentity(playerPOV) }
    }

    fun knows(slotIndex: Int): Boolean {
        return hand.getSlot(slotIndex).isKnown(playerPOV)
    }

}