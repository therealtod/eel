package eelst.ilike.game.entity.slot

import eelst.ilike.game.entity.ClueValue


data class SlotMetadata(
    val index: Int,
    val positiveClues: List<ClueValue> = emptyList(),
    val negativeClues: List<ClueValue> = emptyList(),
)
