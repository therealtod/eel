package eelst.ilike.engine.player

import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand

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
