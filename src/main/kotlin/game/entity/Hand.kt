package eelst.ilike.game.entity

import eelst.ilike.game.entity.slot.Slot

/**
 * A collection of [Slot].
 *
 * Slot indexes are 1-based
 */
interface Hand : List<Slot> {
    /**
     * @return the [Slot] which occupies slot [slotIndex] in this [Hand]
     *
     * Slot indexes are 1-based
     */
    fun getSlot(slotIndex: Int): Slot

    /**
     * @return a [Hand] created from this [Hand] with the given [slot] in first position (slotIndex 1)
     */
    fun withNewSlot(slot: Slot): Hand
}
