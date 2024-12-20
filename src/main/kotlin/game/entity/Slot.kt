package eelst.ilike.game.entity

import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.card.HanabiCard

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
    fun getUpdatedEmpathy(clueValue: ClueValue): Set<HanabiCard>
}
