package eelst.ilike.engine.knowledge

import eelst.ilike.engine.signal.Signal
import eelst.ilike.game.entity.HanabiCard

/**
 * Represents the knowledge that the slot owner has on a slot of theirs
 */
interface SlotKnowledge {
    /**
     * Merge this knowledge with the provided [otherKnowledge]
     *
     * In order for this to work as intended, [otherKnowledge] should be about the same slot
     */
    fun integrateWith(otherKnowledge: SlotKnowledge): SlotKnowledge

    /**
     * @return true if the slot owner has full empathy over the slot
     */
    fun hasFullEmpathy(): Boolean

    /**
     * Get the set of identities that the slot owner can attribute to the slot
     */
    fun getImpliedIdentities(): Set<HanabiCard>

    /**
     * Get the list of [Signal] associated to the slot
     */
    fun getSignals(): Map<Int, Signal>

    /**
     * @return true if the slot owner has received conflicting information about the slot
     */
    fun hasConflictingInformation(): Boolean
}
