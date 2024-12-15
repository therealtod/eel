package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

class FullEmpathySlot(
    globallyAvailableInfo: GloballyAvailableSlotInfo,
    knowledge: PersonalSlotKnowledge,
    identity: HanabiCard,
) : KnownSlot(
    globallyAvailableInfo = globallyAvailableInfo,
    knowledge = knowledge,
    knownIdentity = identity,
    )
