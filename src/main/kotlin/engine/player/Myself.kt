package eelst.ilike.engine.player

import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot

class Myself(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    override val hand: Hand,
): Teammate(
    globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
    hand = hand
) {
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex

    override fun getSlots() = hand.getSlots()
}
