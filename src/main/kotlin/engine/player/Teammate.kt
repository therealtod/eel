package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class Teammate(
    override val playerId: PlayerId,
    override val playerIndex: Int,
    personalKnowledge: PersonalKnowledge,
    val hand: InterpretedHand,
    val seatsGap: Int,
) : Player<InterpretedSlot> {
    fun playsBefore(otherTeammate: Teammate): Boolean {
        return seatsGap < otherTeammate.seatsGap
    }

    fun getSlotFromTeammatePOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    fun getOwnKnownCards(): List<HanabiCard>
}
