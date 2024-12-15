package eelst.ilike.engine.player

import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand

class POVProjectionAsTeammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PlayerPersonalKnowledge,
    hand: Hand
) : Teammate(
    globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
    personalKnowledge = personalKnowledge,
    hand = hand,
)
