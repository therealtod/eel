package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo

object HandFactory {
    fun createOwnSlots(
        handSize: Int,
        playerGlobalInfo: GloballyAvailablePlayerInfo,
        personalSlotKnowledge: Set<PersonalSlotKnowledge>,
    ): Set<OwnSlot> {
        return (1..handSize).map {
            OwnSlot(
                globalInfo = playerGlobalInfo.hand.elementAt(it - 1),
                slotKnowledge = personalSlotKnowledge.elementAt(it - 1)
            )
        }.toSet()
    }

    fun createOwnHand(
        handSize: Int,
        playerGlobalInfo: GloballyAvailablePlayerInfo,
        personalSlotKnowledge: Set<PersonalSlotKnowledge>,
    ): OwnHand {
        val slots = createOwnSlots(
            handSize = handSize,
            playerGlobalInfo = playerGlobalInfo,
            personalSlotKnowledge = personalSlotKnowledge
        )
        return OwnHand(slots)
    }
}