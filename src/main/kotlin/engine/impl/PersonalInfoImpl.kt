package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.engine.TeammatePersonalInfo
import eelst.ilike.game.PlayerId

data class PersonalInfoImpl(
    val ownHandInfo: Set<PersonalSlotInfo> = emptySet(),
    val teammates: Map<PlayerId, TeammatePersonalInfo> = emptyMap()
): PersonalInfo {
    override fun getOwnSlotInfo(slotIndex: Int): PersonalSlotInfo {
        return ownHandInfo.elementAt(slotIndex - 1 )
    }

    override fun getTeammateHand(playerId: PlayerId): TeammateHand {
        val teammate = teammates[playerId]
            ?: throw IllegalArgumentException("No player with id $playerId that I know of")
        val slots = teammate.slots
        return TeammateHand(slots)
    }

    override fun getTeammatePersonalInfo(playerId: PlayerId): TeammatePersonalInfo {
        return teammates[playerId]
            ?: throw IllegalArgumentException("No player with id $playerId that I know of")
    }
}
