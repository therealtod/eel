package eelst.ilike.game.entity

import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

interface Slot {
    val index: Int
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>
    fun getPossibleIdentities(): Set<HanabiCard>
    fun isTouched(): Boolean
    fun containsCard(card: HanabiCard): Boolean
    fun matches(condition: (slotIndex: Int, card: HanabiCard) -> Boolean): Boolean
}
