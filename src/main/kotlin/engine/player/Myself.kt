package eelst.ilike.engine.player

import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.entity.Hand

class Myself(
    playerMetadata: PlayerMetadata,
    override val hand: Hand,
): Teammate(
    playerMetadata = playerMetadata,
    hand = hand
) {
    override val playerIndex = playerMetadata.playerIndex

    override fun getSlots() = hand.getSlots()
}
