package eelst.ilike.game.entity

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class SimpleSlot(
    globallyAvailableSlotInfo: GloballyAvailableSlotInfo
): Slot {
    override val index = globallyAvailableSlotInfo.index
    override val positiveClues = globallyAvailableSlotInfo.positiveClues
    override val negativeClues = globallyAvailableSlotInfo.negativeClues

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    override fun getEmpathy(visibleCards: List<HanabiCard>, suites: Set<Suite>): Set<HanabiCard> {
        return GameUtils.getCardEmpathy(
            visibleCards = visibleCards,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            suites = suites
        )
    }

    override fun containsCard(card: HanabiCard): Boolean {
        TODO()
    }
}
