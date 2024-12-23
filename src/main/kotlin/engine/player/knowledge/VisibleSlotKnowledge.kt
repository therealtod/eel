package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.convention.hgroup.signal.Signal
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlotKnowledge(
    val visibleCard: HanabiCard,
    signals: MutableMap<Int, Signal>,
    impliedIdentities: Set<HanabiCard>,
    hasConflictingInformation: Boolean,
): DefaultSlotKnowledge(
    signals = signals,
    impliedIdentities = impliedIdentities,
    hasConflictingInformation = hasConflictingInformation
)
{
    override fun isVisible(): Boolean {
        return true
    }
}
