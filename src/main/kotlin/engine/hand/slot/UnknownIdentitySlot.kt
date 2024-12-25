package eelst.ilike.engine.hand.slot

import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

class UnknownIdentitySlot(
    slotMetadata: SlotMetadata,
    possibleIdentities: Set<HanabiCard>,
): BaseSlot(
    globallyAvailableInfo = slotMetadata,
    possibleIdentities = possibleIdentities
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return false
    }

    override fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean {
        return false
    }

    override fun isTouchedBy(clueValue: ClueValue): Boolean {
        return getPossibleIdentities().all { it.isTouchedBy(clueValue) }
    }
}
