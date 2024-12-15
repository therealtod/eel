package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

class UnknownIdentitySlot(
    globallyAvailableInfo: GloballyAvailableSlotInfo,
    knowledge: PersonalSlotKnowledge,
): BaseSlot(
    globallyAvailableInfo = globallyAvailableInfo,
    knowledge = knowledge,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return false
    }

    override fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean {
        return false
    }
}
