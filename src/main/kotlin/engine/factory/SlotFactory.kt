package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.game.entity.Slot

object SlotFactory {
    fun createSlot(
        globalInfo: GloballyAvailableSlotInfo,
        knowledge: PersonalSlotKnowledge,
    ): Slot {
        return if (knowledge.isSlotKnown()) {
            KnownSlot(
                globallyAvailableInfo = globalInfo,
                knowledge = knowledge,
                knownIdentity = knowledge.getPossibleSlotIdentities().first()
            )
        } else {
            UnknownIdentitySlot(
                globallyAvailableInfo = globalInfo,
                knowledge = knowledge
            )
        }
    }
}
