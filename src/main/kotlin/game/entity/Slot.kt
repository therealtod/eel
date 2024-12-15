package eelst.ilike.game.entity

import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

interface Slot {
    val index: Int
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>
    fun getGloballyAvailableInfo(): GloballyAvailableSlotInfo
    fun getPossibleIdentities(): Set<HanabiCard>
    fun isTouched(): Boolean
    fun isTouchedBy(clueValue: ClueValue): Boolean
    fun containsCard(card: HanabiCard): Boolean
    fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean
    fun getUpdatedEmpathy(clueValue: ClueValue): Set<HanabiCard>
}
