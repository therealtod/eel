package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class VisibleHand(private val slots: Set<VisibleSlot>) : InterpretedHand, Set<InterpretedSlot> by slots {
    override val size = slots.size

    override fun copiesOf(card: HanabiCard, playerPOV: PlayerPOV): Int {
        return slots.count { it.card == card }
    }

    override fun holds(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        return slots.any { it.card == card }
    }

    override fun getSlotsTouchedBy(clueValue: ClueValue, playerPOV: PlayerPOV): Set<Slot> {
        return slots.filter { it.card.suite.clueTouches(it.card, clueValue) }.toSet()
    }

    override fun getSlot(slotIndex: Int): VisibleSlot {
        return slots.elementAt(slotIndex - 1)
    }

    override fun getSlots(): Set<VisibleSlot> {
        return slots
    }

    override fun isVisibleFrom(playerPOV: PlayerPOV): Boolean {
        return true
    }

    override fun getAsVisible(): VisibleHand {
        return this
    }

    fun getCardinSlot(slotIndex: Int): HanabiCard {
        return slots.elementAt(slotIndex - 1).card
    }
}