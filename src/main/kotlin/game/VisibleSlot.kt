package eelst.ilike.game

import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.entity.card.HanabiCard

data class VisibleSlot(
    val globalInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
): InterpretedSlot(
    globalInfo = globalInfo,
) {
    fun fromOwnerPOV(personalSlotInfo: PersonalSlotInfo): OwnSlot {
        return OwnSlot(
            globalInfo = globalInfo,
            impliedIdentities = personalSlotInfo.impliedIdentities
        )
    }
}
