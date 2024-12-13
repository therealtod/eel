package eelst.ilike.game.entity

import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

interface Slot {
    val index: Int
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>
    fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard>
    fun isTouched(): Boolean
    fun containsCard(card: HanabiCard): Boolean
}
