package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PlayerKnowledge
import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

class UnknownIdentitySlot(
    slotMetadata: SlotMetadata,
    knowledge: PlayerKnowledge,
): BaseSlot(
    globallyAvailableInfo = slotMetadata,
    knowledge = knowledge,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return false
    }

    override fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean {
        return false
    }

    override fun isTouchedBy(clueValue: ClueValue): Boolean {
        // return knowledge.getPossibleSlotIdentities().all { it.isTouchedBy(clueValue) }
        TODO()
    }
}
