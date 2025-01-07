package eelst.ilike.game.entity.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard

/**
 * Represents a Player's slot
 */
interface Slot {
    val index: Int
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>
    fun getGloballyAvailableInfo(): SlotMetadata
    fun getPossibleIdentities(): Set<HanabiCard>
    fun isTouched(): Boolean
    fun isTouchedBy(clueValue: ClueValue): Boolean
    fun containsCard(card: HanabiCard): Boolean
    fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean
}
