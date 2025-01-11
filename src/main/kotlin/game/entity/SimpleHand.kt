package eelst.ilike.game.entity

import eelst.ilike.game.entity.slot.Slot

class SimpleHand(
    private val slots: List<Slot>
) : Hand, List<Slot> by slots {
    override val size = slots.size

    override fun getSlot(slotIndex: Int): Slot {
        return slots[slotIndex - 1]
    }

    override fun withNewSlot(slot: Slot): Hand {
        TODO("Not yet implemented")
    }
}