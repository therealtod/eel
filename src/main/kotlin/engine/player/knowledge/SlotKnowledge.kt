package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.convention.hgroup.signal.Signal
import eelst.ilike.game.entity.card.HanabiCard




interface SlotKnowledge {
    fun integrateWith(otherKnowledge: SlotKnowledge): SlotKnowledge
    fun isVisible(): Boolean
    fun getImpliedIdentities(): Set<HanabiCard>
    fun getSignals(): Map<Int, Signal>
    fun hasConflictingInformation(): Boolean
    fun asNotVisible(): SlotKnowledge
    fun getIdentity(): HanabiCard
}
