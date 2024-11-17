package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

data class SlotKnowledge(
    val globalInfo: GloballyAvailableSlotInfo,
    val impliedIdentities: Set<HanabiCard>,
)
