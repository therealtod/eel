package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

class VisibleSlot(
    globallyAvailableInfo: GloballyAvailableSlotInfo,
    knowledge: PersonalSlotKnowledge,
    visibleCard: HanabiCard,
): KnownSlot(
    globallyAvailableInfo = globallyAvailableInfo,
    knowledge = knowledge,
    knownIdentity = visibleCard,
)
