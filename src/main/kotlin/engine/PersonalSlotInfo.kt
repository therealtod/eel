package eelst.ilike.engine

import eelst.ilike.game.entity.card.HanabiCard

data class PersonalSlotInfo(
    val slotIndex: Int,
    val impliedIdentities: Set<HanabiCard> = emptySet(),
)
