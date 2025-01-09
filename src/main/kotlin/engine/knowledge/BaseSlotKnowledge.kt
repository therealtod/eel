package eelst.ilike.engine.knowledge

import eelst.ilike.engine.signal.Signal
import eelst.ilike.game.entity.HanabiCard

open class BaseSlotKnowledge(
    private val signals: MutableMap<Int, Signal> = mutableMapOf(),
    private var impliedIdentities: Set<HanabiCard> = emptySet(),
    private var hasConflictingInformation: Boolean = false,
): SlotKnowledge {
    override fun hasFullEmpathy(): Boolean {
        return false
    }

    override fun integrateWith(otherKnowledge: SlotKnowledge): SlotKnowledge {
        if (otherKnowledge.hasFullEmpathy()) {
            return otherKnowledge.integrateWith(this)
        }
        signals.putAll(otherKnowledge.getSignals())
        val updatedImpliedIdentities = impliedIdentities.intersect(otherKnowledge.getImpliedIdentities())
        hasConflictingInformation = impliedIdentities.isNotEmpty()
                && otherKnowledge.getImpliedIdentities().isNotEmpty()
                && updatedImpliedIdentities.isEmpty()
        impliedIdentities = updatedImpliedIdentities
        return this
    }

    override fun getImpliedIdentities(): Set<HanabiCard> {
        return impliedIdentities
    }

    override fun getSignals(): Map<Int, Signal> {
        return signals
    }

    override fun hasConflictingInformation(): Boolean {
        return hasConflictingInformation
    }
}
