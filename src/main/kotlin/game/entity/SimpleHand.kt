package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class SimpleHand(
    override val ownerId: PlayerId,
    private val slots: Set<Slot>
): Hand, Set<Slot> by slots {
    override fun getSlots(): Set<Slot> {
        return slots
    }

    override fun getSlot(slotIndex: Int): Slot {
        require(slotIndex in 1..size) {
            "$slotIndex is not a valid slot index"
        }
        return slots.elementAt(slotIndex - 1)
    }

    override fun getSlotsTouchedBy(clueValue: ClueValue): Set<Int> {
        return slots.filter { it.isTouchedBy(clueValue) }.map { it.index }.toSet()
    }

    override fun countCopiesOf(card: HanabiCard): Int {
        return slots.count { it.containsCard(card) }
    }
}
