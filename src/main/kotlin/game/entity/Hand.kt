package eelst.ilike.game.entity

import eelst.ilike.game.entity.slot.Slot

interface Hand : List<Slot> {
    fun getSlots(): List<Slot>
    fun getSlot(slotIndex: Int): Slot
    fun getSlotsTouchedBy(clueValue: ClueValue): Set<Int>
    fun countCopiesOf(card: HanabiCard): Int
    fun withNewSlot(slot: Slot): Hand
}
