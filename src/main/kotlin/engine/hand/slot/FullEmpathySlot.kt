package eelst.ilike.engine.hand.slot

import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.card.HanabiCard

class FullEmpathySlot(
    globallyAvailableInfo: SlotMetadata,
    identity: HanabiCard,
) : KnownSlot(
    globallyAvailableInfo = globallyAvailableInfo,
    knownIdentity = identity,
    )
