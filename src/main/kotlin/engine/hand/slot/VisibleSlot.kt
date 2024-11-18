package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    globalInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
) : InterpretedSlot(
    globalInfo = globalInfo,
) {
    override fun getAsKnown(): KnownSlot {
        return KnownSlot(
            globallyAvailableInfo = globalInfo,
            card = card,
        )
    }
}
