package eelst.ilike.engine.knowledge

import eelst.ilike.engine.signal.Signal
import eelst.ilike.game.entity.HanabiCard

open class BaseSlotKnowledge(
    private val slotOwnerPlayerIndex: Int,
    /**
     * The possible [HanabiCard] identities that the slot owner attributes to this slot
     */
    private val impliedIdentities: Set<HanabiCard> = emptySet(),
    private val hasConflictingInformation: Boolean = false,
    private val signals: List<List<Signal>> = emptyList(),
    empathyPerPlayer: List<Set<HanabiCard>>,
) : SlotKnowledge {
    private val empathyPerPlayer: List<Set<HanabiCard>> by lazy { empathyPerPlayer }

    override fun hasFullEmpathy(playerIndex: Int): Boolean {
        return empathyPerPlayer[playerIndex].size == 1
    }

    override fun getEmpathy(playerIndex: Int): Set<HanabiCard> {
        return empathyPerPlayer[playerIndex]
    }

    override fun integrateWith(otherKnowledge: SlotKnowledge): SlotKnowledge {
        val updatedImpliedIdentities = impliedIdentities.intersect(otherKnowledge.getImpliedIdentities())
        val isConflict = impliedIdentities.isNotEmpty()
                && otherKnowledge.getImpliedIdentities().isNotEmpty()
                && updatedImpliedIdentities.isEmpty()
        return BaseSlotKnowledge(
            slotOwnerPlayerIndex = slotOwnerPlayerIndex,
            impliedIdentities = updatedImpliedIdentities,
            hasConflictingInformation = isConflict,
            signals = signals.mapIndexed { index, signals ->
                signals + otherKnowledge.getSignalsPerceivedBy(index)
            },
            empathyPerPlayer = empathyPerPlayer.mapIndexed { index, empathy ->
                empathy.intersect(otherKnowledge.getEmpathy(index))
            }
        )
    }

    override fun getImpliedIdentities(): Set<HanabiCard> {
        return impliedIdentities
    }

    override fun getSignals(): List<List<Signal>> {
        return signals
    }

    override fun getSignalsPerceivedBy(playerIndex: Int): List<Signal> {
        return signals[playerIndex]
    }

    override fun hasConflictingInformation(): Boolean {
        return hasConflictingInformation
    }

    override fun slotIsKnownByOwner(): Boolean {
        return impliedIdentities.size == 1 || empathyPerPlayer[slotOwnerPlayerIndex].size == 1
    }

    override fun getInferredIdentity(): HanabiCard {
        if (!slotIsKnownByOwner()) {
            throw IllegalAccessException("Cannot get the inferred identity of a slot which is not known by the owner")
        }
        else {
            return impliedIdentities.ifEmpty { empathyPerPlayer[slotOwnerPlayerIndex] }.first()
        }
    }
}
