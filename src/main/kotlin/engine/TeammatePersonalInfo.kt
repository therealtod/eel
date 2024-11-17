package eelst.ilike.engine

import eelst.ilike.game.VisibleSlot

interface TeammatePersonalInfo {
    fun getSlotInfo(slotIndex: Int): PersonalSlotInfo
}
