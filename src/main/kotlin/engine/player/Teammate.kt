package eelst.ilike.engine.player

import eelst.ilike.engine.BasePlayer
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
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
    val hand: TeammateHand,
    val seatsGap: Int,
) : BasePlayer(
    playerId = playerId,
    playerIndex = playerIndex,
) {
    override val ownHand = playerPOV.ownHand
    override val playerPOV: PlayerPOV
        get() = TODO("Not yet implemented")

    fun playsBefore(otherTeammate: Teammate): Boolean {
        return seatsGap < otherTeammate.seatsGap
    }

    override fun getSlots(): Set<InterpretedSlot> {
        return hand.getSlots()
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return hand.getSlot(slotIndex).card == card
    }
}
