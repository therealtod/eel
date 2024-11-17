package eelst.ilike.game.entity

interface Hand<T : Slot> : Set<T> {
    fun getSlot(slotIndex: Int): T
}
