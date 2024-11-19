package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class TeammateHand(val slots: Set<VisibleSlot>) : InterpretedHand, Set<Slot> by slots{
    override val size = slots.size

    override fun copiesOf(card: HanabiCard): Int {
        return slots.count { it.card == card }
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        return slots.filter { clue.touches(it.card) }.toSet()
    }

    override fun getKnownSlots(): Set<KnownSlot> {
        return slots.map { it.getAsKnown() }.toSet()
    }

    override fun getSlot(slotIndex: Int): VisibleSlot {
        return slots.elementAt(slotIndex - 1)
    }
}