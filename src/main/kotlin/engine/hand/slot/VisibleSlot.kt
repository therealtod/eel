package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class VisibleSlot(
    index: Int,
    positiveClues: List<ClueValue>,
    negativeClues: List<ClueValue>,
    val visibleCard: HanabiCard,
): BaseSlot(
    index = index,
    positiveClues = positiveClues,
    negativeClues =  negativeClues,
) {
    constructor(globallyAvailableSlotInfo: GloballyAvailableSlotInfo, visibleCard: HanabiCard)
            : this(
                    index = globallyAvailableSlotInfo.index,
                    positiveClues = globallyAvailableSlotInfo.positiveClues,
                    negativeClues = globallyAvailableSlotInfo.negativeClues,
                    visibleCard = visibleCard,
            )

    override fun containsCard(card: HanabiCard): Boolean {
        return card == visibleCard
    }
}
