package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

open class KnownSlot(
    globallyAvailableInfo: GloballyAvailableSlotInfo,
    knowledge: PersonalSlotKnowledge,
    val knownIdentity: HanabiCard,
): BaseSlot(
    globallyAvailableInfo = globallyAvailableInfo,
    knowledge = knowledge,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return card == knownIdentity
    }

    override fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean {
        return condition.invoke(index, knownIdentity)
    }
}
