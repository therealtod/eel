package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

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
