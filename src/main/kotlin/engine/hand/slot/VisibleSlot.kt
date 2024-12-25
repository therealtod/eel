package eelst.ilike.engine.hand.slot

import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    slotMetadata: SlotMetadata,
    visibleCard: HanabiCard,
): KnownSlot(
    globallyAvailableInfo = slotMetadata,
    knownIdentity = visibleCard,
)
