package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

data class KnownSlot(
    val globallyAvailableInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
): InterpretedSlot(
    globalInfo = globallyAvailableInfo
)
