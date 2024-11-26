package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class VisibleHand(private val slots: Set<VisibleSlot>) : InterpretedHand, Set<InterpretedSlot> by slots {
    override val size = slots.size

    override fun copiesOf(card: HanabiCard): Int {
        return slots.count { it.card == card }
    }

    override fun getSlotsTouchedBy(clueValue: ClueValue): Set<Slot> {
        return slots.filter { it.card.suite.clueTouches(it.card, clueValue) }.toSet()
    }

    override fun getKnownSlots(): Set<KnownSlot> {
        return slots.map { it.asKnown() }.toSet()
    }

    override fun getSlot(slotIndex: Int): VisibleSlot {
        return slots.elementAt(slotIndex - 1)
    }

    override fun getSlots(): Set<VisibleSlot> {
        return slots
    }
}