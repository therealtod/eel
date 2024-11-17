package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.Utils
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

abstract class InterpretedSlot(
    val globalInfo: GloballyAvailableSlotInfo,
): Slot {
    override val index = globalInfo.index
    override val positiveClues = globalInfo.positiveClues
    override val negativeClues = globalInfo.negativeClues

    override fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard> {
        return Utils.getCardEmpathy(
            visibleCards = visibleCards,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            suites = suites
        )
    }

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    open fun isClued(): Boolean {
        return positiveClues.isNotEmpty()
    }
}