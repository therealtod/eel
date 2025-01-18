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
     * @return the empathy associated with the slot as seen by the player with the given [playerIndex]
     */
    fun getEmpathy(playerIndex: Int): Set<HanabiCard>

    /**
     * @return true if the slot owner has full empathy over the slot as seen by the player with the given [playerIndex]
     */
    fun hasFullEmpathy(playerIndex: Int): Boolean

    /**
     * Get the set of identities that the slot owner can attribute to the slot
     */
    fun getImpliedIdentities(): Set<HanabiCard>

    /**
     * Get the signals that each player perceived has been given to this slot
     */
    fun getSignals(): List<List<Signal>>

    /**
     * Get the list of [Signal] associated to the slot as seen by the player with the given [playerIndex]
     */
    fun getSignalsPerceivedBy(playerIndex: Int): List<Signal>

    /**
     * @return true if the slot owner has received conflicting information about the slot
     */
    fun hasConflictingInformation(): Boolean

    /**
     * @return true if the owner thinks they know the identity fo this slot
     */
    fun slotIsKnownByOwner(): Boolean

    /**
     * @return the identity of this slot according to the slot owner
     *
     * @throws [IllegalAccessException] if the slot is not known
     */
    fun getInferredIdentity(): HanabiCard
}
