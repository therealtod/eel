package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

data class KnownSlot(
    val globallyAvailableInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
) : InterpretedSlot(
    globalInfo = globallyAvailableInfo
) {
    override fun asKnown(): KnownSlot {
        return this
    }

    override fun contains(card: HanabiCard): Boolean {
        return this.card == card
    }

    override fun isKnown(): Boolean {
        return true
    }

    override fun getPossibleIdentities(): Set<HanabiCard> {
        return setOf(card)
    }
}
