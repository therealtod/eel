package eelst.ilike.engine

import eelst.ilike.engine.impl.TeammateHand
import eelst.ilike.game.PlayerId

interface PersonalInfo {
    fun getOwnSlotInfo(slotIndex: Int): PersonalSlotInfo

    fun getTeammateHand(playerId: PlayerId): TeammateHand
}
