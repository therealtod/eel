package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.SimpleSlot
import eelst.ilike.game.entity.Slot

object SlotFactory {
    fun createSlot(
        globalInfo: GloballyAvailableSlotInfo,
        knowledge: PersonalSlotKnowledge,
    ): Slot {
        return if (knowledge.isSlotKnown()) {
            KnownSlot(
                index = globalInfo.index,
                positiveClues = globalInfo.positiveClues,
                negativeClues = globalInfo.negativeClues,
                knownIdentity = knowledge.getPossibleSlotIdentities().first()
            )
        } else {
            SimpleSlot(
                globallyAvailableSlotInfo = globalInfo
            )
        }
    }
}
