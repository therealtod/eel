package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.engine.InfoOnTeammate
import eelst.ilike.game.PlayerId

data class PersonalInfoImpl(
    val ownHandInfo: Set<PersonalSlotInfo> = emptySet(),
    val teammates: Map<PlayerId, InfoOnTeammate> = emptyMap()
): PersonalInfo {
    override fun getOwnSlotInfo(slotIndex: Int): PersonalSlotInfo {
        return ownHandInfo.elementAt(slotIndex - 1)
    }

    /*
    override fun getTeammateHand(playerId: PlayerId): TeammateHand {
        val teammate = teammates[playerId]
            ?: throw IllegalArgumentException("No player with id $playerId that I know of")
        val slots = teammate.
        return TeammateHand(slots)
    }

     */

    override fun getInfo(playerId: PlayerId): InfoOnTeammate {
        return teammates[playerId]
            ?: throw IllegalArgumentException("No player with id $playerId that I know of")
    }
}
