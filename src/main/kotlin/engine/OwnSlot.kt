package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.Utils
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class OwnSlot(
    globalInfo: GloballyAvailableSlotInfo,
    private val slotKnowledge: PersonalSlotKnowledge,
) : InterpretedSlot(globalInfo) {
    fun getPossibleIdentities(): Set<HanabiCard> {
        return slotKnowledge.getPossibleSlotIdentities()
    }

    fun hasKnownIdentity(card: HanabiCard): Boolean {
        return isKnown() && getPossibleIdentities().first() == card
    }

    fun isKnown(): Boolean {
        return getPossibleIdentities().size == 1
    }

    override fun isClued(): Boolean {
        return slotKnowledge.isClued()
    }
}
