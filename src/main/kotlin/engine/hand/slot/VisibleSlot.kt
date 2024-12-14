package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.BaseSlot
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    index: Int,
    positiveClues: List<ClueValue>,
    negativeClues: List<ClueValue>,
    visibleCard: HanabiCard,
): KnownSlot(
    index = index,
    positiveClues = positiveClues,
    negativeClues =  negativeClues,
    knownIdentity = visibleCard
) {
    constructor(globallyAvailableSlotInfo: GloballyAvailableSlotInfo, visibleCard: HanabiCard)
            : this(
                    index = globallyAvailableSlotInfo.index,
                    positiveClues = globallyAvailableSlotInfo.positiveClues,
                    negativeClues = globallyAvailableSlotInfo.negativeClues,
                    visibleCard = visibleCard,
            )
}
