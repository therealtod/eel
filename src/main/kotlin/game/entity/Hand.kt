package eelst.ilike.game.entity

interface Hand: Set<Slot> {
    fun getSlot(slotIndex: Int): Slot
}
