package eelst.ilike.engine.knowledge

/**
 * Represent the aggregated knowledge that the owner of a Hand has about it
 */
interface HandKnowledge {
    /**
     * Merge the information contained in this object with new knowledge
     *
     * In order for this to work properly [otherKnowledge] should refer to the hand of the same player
     */
    fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge
}
