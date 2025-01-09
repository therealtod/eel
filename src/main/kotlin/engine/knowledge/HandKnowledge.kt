package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.HanabiCard

/**
 * Represent the aggregated knowledge that the owner a Hand has about it
 */
interface HandKnowledge {
    /**
     * Merge the information contained in this object with new knowledge
     *
     * In order for this to work properly [otherKnowledge] should refer to the hand of the same player
     */
    fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge

    /**
     * Get the [SlotKnowledge] that the hand owner has about the slot with the given [slotIndex]
     */
    fun getSlotKnowledge(slotIndex: Int): SlotKnowledge

    /**
     * @return the set of indexes of slot contained in this hand and whose identity should be known to the hand owner
     */
    fun getKnownSlotIndexes(): Collection<Int>
}
