package eelst.ilike.game

import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    globalInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
): InterpretedSlot(
    globalInfo = globalInfo,
)
