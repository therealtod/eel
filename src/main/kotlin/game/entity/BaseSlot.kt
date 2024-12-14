package eelst.ilike.game.entity

import eelst.ilike.game.GameUtils
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

abstract class BaseSlot(
    override val index: Int,
    override val positiveClues: List<ClueValue>,
    override val negativeClues: List<ClueValue>,
): Slot {
    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    override fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard> {
        return GameUtils.getCardEmpathy(
            visibleCards = visibleCards,
            suites = suites,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
        )
    }
}
