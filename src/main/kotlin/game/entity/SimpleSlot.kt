package eelst.ilike.game.entity

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class SimpleSlot(
    globallyAvailableSlotInfo: GloballyAvailableSlotInfo
): BaseSlot(
    index = globallyAvailableSlotInfo.index,
    positiveClues = globallyAvailableSlotInfo.positiveClues,
    negativeClues = globallyAvailableSlotInfo.negativeClues,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return false
    }
}
