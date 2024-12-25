package eelst.ilike.engine.factory

import eelst.ilike.game.SlotMetadata
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.SlotKnowledge
import eelst.ilike.engine.player.knowledge.TeamKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suit

object SlotFactory {
    fun createSlot(
        slotData: SlotMetadata,
        slotKnowledge: SlotKnowledge,
        cardsVisibleBySlotOwner: List<HanabiCard>,
        suits: Set<Suit>,
    ): Slot {
        return if(slotKnowledge.isVisible()) {
            VisibleSlot(
                slotMetadata = slotData,
                visibleCard = slotKnowledge.getIdentity()
            )
        } else {
            UnknownIdentitySlot(
                slotMetadata = slotData,
                possibleIdentities = slotKnowledge.getImpliedIdentities()
                    .ifEmpty { GameUtils.getCardEmpathy(
                        visibleCards = cardsVisibleBySlotOwner,
                        positiveClues = slotData.positiveClues,
                        negativeClues = slotData.negativeClues,
                        suits = suits,
                    ) }
            )
        }
        /*
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

         */
    }
}
