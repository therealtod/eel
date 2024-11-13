package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.engine.PersonalTeammateInfo
import eelst.ilike.game.PlayerId

data class PersonalInfoImpl(
    val ownHandInfo: Set<PersonalSlotInfo> = emptySet(),
    val teammates: Map<PlayerId, PersonalTeammateInfo> = emptyMap()
): PersonalInfo {
    override fun getOwnSlotInfo(slotIndex: Int): PersonalSlotInfo {
        return ownHandInfo.elementAt(slotIndex - 1 )
    }

    override fun getTeammateHand(playerId: PlayerId): TeammateHand {
        TODO("Not yet implemented")
    }
}
