package eelst.ilike.engine.impl


import eelst.ilike.engine.InfoOnTeammate
import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.PersonalSlotInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.VisibleSlot

data class InfoOnTeammateImpl(
    val slots: Set<VisibleSlot>,
    val teammatePOVInfo: Set<PersonalSlotInfo>
): InfoOnTeammate {
    override fun getSlot(slotIndex: Int, slotGlobalInfo: GloballyAvailableSlotInfo): VisibleSlot {
        return VisibleSlot(
            globalInfo = slotGlobalInfo,
            card = slots.elementAt(slotIndex - 1).card
        )
    }

    override fun getOwnInfo(): PersonalInfo {
        val slotInfo = slots.map {
            PersonalSlotInfo(
                slotIndex = it.index,
                impliedIdentities = emptySet()
            )
        }
        return PersonalInfoImpl(
            ownHandInfo = slotInfo.toSet()
        )
    }
}
