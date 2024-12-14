package eelst.ilike.engine.hand.slot

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

class FullEmpathySlot(
    globallyAvailableSlotInfo: GloballyAvailableSlotInfo,
    identity: HanabiCard,
) : KnownSlot(
        index = globallyAvailableSlotInfo.index,
        positiveClues = globallyAvailableSlotInfo.positiveClues,
        negativeClues = globallyAvailableSlotInfo.negativeClues,
        knownIdentity = identity,
    )
