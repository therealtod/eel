package eelst.ilike.game.entity

import eelst.ilike.engine.action.Clue
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

interface Slot {
    val index: Int
    val positiveClues: List<Clue>
    val negativeClues: List<Clue>
    fun isTouched(): Boolean
    fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard>
}
