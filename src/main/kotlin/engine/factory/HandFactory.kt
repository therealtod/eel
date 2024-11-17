package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo

object HandFactory {
    fun createOwnSlots(
        handSize: Int,
        playerGlobalInfo: GloballyAvailablePlayerInfo,
        personalHandKnowledge: PersonalHandKnowledge,
    ): Set<OwnSlot> {
        return (1..handSize).map {
            OwnSlot(
                globalInfo = playerGlobalInfo.hand.elementAt(it - 1),
                slotKnowledge = personalHandKnowledge.getKnowledge(it)
            )
        }.toSet()
    }

    fun createOwnHand(
        handSize: Int,
        playerGlobalInfo: GloballyAvailablePlayerInfo,
        personalHandKnowledge: PersonalHandKnowledge,
    ): OwnHand {
        val slots = createOwnSlots(
            handSize = handSize,
            playerGlobalInfo = playerGlobalInfo,
            personalHandKnowledge = personalHandKnowledge
        )
        return OwnHand(slots)
    }
}