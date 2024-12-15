package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.utils.Configuration
import eelst.ilike.utils.InputParser.parseCard
import eelst.ilike.utils.model.dto.SlotDTO

object SlotFactory {
    fun createSlot(
        activePlayerId: PlayerId,
        slotOwnerId: PlayerId,
        globallyAvailableSlotInfo: GloballyAvailableSlotInfo,
        knowledge: PersonalSlotKnowledge,
        visibleIdentity: HanabiCard?,
    ): Slot {
        return if (visibleIdentity != null) {
            if (activePlayerId == slotOwnerId) {
                FullEmpathySlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                    identity = visibleIdentity
                )
            } else {
                VisibleSlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                    visibleCard = visibleIdentity
                )
            }
        } else {
            if (knowledge.isSlotKnown()) {
                KnownSlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                    knownIdentity = knowledge.getPossibleSlotIdentities().first()
                )
            } else {
                UnknownIdentitySlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                )
            }
        }
    }
}