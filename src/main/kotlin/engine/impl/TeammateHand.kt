package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.Slot
import eelst.ilike.game.VisibleSlot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class TeammateHand(val slots: Set<VisibleSlot>): InterpretedHand, Set<InterpretedSlot> by slots {
    override val size = slots.size

    override fun copiesOf(card: HanabiCard): Int {
        return slots.count { it.card == card }
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        return slots.filter { clue.touches(it.card) }.toSet()
    }

    fun getCards(): List<HanabiCard> {
        return slots.map { it.card}
    }

    fun getSlot(slotIndex: Int): VisibleSlot {
        return slots.elementAt(slotIndex - 1 )
    }
}