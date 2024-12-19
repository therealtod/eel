package eelst.ilike.hanablive.model

import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player

interface HanabLivePlayer: Player {
    override val hand: HanabLiveHand
}
