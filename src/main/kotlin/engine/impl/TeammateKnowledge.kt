package eelst.ilike.engine.impl

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

class TeammateKnowledge(
    globallyAvailableInfo: GloballyAvailableInfo,
    teammates: Set<Teammate>,
    hand: TeammateHand,
)
