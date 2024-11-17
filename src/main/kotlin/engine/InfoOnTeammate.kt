package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.VisibleSlot

interface InfoOnTeammate {
    fun getSlot(slotIndex: Int, slotGlobalInfo: GloballyAvailableSlotInfo): VisibleSlot
    fun getOwnInfo(): PersonalInfo
}
