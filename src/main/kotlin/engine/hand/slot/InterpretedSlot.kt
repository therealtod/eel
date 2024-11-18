package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

abstract class InterpretedSlot(
    val globalInfo: GloballyAvailableSlotInfo,
) : Slot {
    override val index = globalInfo.index
    override val positiveClues = globalInfo.positiveClues
    override val negativeClues = globalInfo.negativeClues

    override fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard> {
        return GameUtils.getCardEmpathy(
            visibleCards = visibleCards,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            suites = suites
        )
    }

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    abstract fun getAsKnown(): KnownSlot

    open fun isClued(): Boolean {
        return positiveClues.isNotEmpty()
    }
}