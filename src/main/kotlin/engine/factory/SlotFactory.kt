package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.SlotMetadata
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PlayerKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

object SlotFactory {
    fun createSlot(
        activePlayerId: PlayerId,
        slotOwnerId: PlayerId,
        slotMetadata: SlotMetadata,
        knowledge: PlayerKnowledge,
        visibleIdentity: HanabiCard?,
    ): Slot {
        return if (visibleIdentity != null) {
            if (activePlayerId == slotOwnerId) {
                FullEmpathySlot(
                    globallyAvailableInfo = slotMetadata,
                    knowledge = knowledge,
                    identity = visibleIdentity
                )
            } else {
                VisibleSlot(
                    slotMetadata = slotMetadata,
                    knowledge = knowledge,
                    visibleCard = visibleIdentity
                )
            }
        } else {
            if (TODO()) {
                KnownSlot(
                    globallyAvailableInfo = slotMetadata,
                    knowledge = knowledge,
                    knownIdentity = TODO()
                )
            } else {
                UnknownIdentitySlot(
                    slotMetadata = slotMetadata,
                    knowledge = knowledge,
                )
            }
        }
    }
}
