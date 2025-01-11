package eelst.ilike.hanablive.entity

import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.slot.Slot
import eelst.ilike.hanablive.entity.slot.HanabLiveSlot

class HanabLiveHand(
    private val slots: Map<HanabLiveCardIndex, HanabLiveSlot>
) : Hand, List<Slot> by slots.values.toList() {
    override val size = slots.size

    override fun getSlot(slotIndex: Int): Slot {
        return slots.values.first { it.index == slotIndex }
    }

    override fun withNewSlot(slot: Slot): Hand {
        TODO("Not yet implemented")
    }
}