package eelst.ilike.hanablive.model

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.BaseHand
import eelst.ilike.game.entity.Hand

class HanabLiveHand(
    ownerId: PlayerId,
    slots: Set<HanabLiveSlot>,
): BaseHand(
    ownerId = ownerId,
    slots = slots,
)
