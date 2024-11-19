package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    globalInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
) : InterpretedSlot(
    globalInfo = globalInfo,
) {
    override fun asKnown(): KnownSlot {
        return KnownSlot(
            globallyAvailableInfo = globalInfo,
            card = card,
        )
    }

    override fun contains(card: HanabiCard): Boolean {
        return this.card == card
    }

    override fun isKnown(): Boolean {
        return true
    }
}
