package eelst.ilike.game

import eelst.ilike.game.entity.ClueValue


data class SlotMetadata(
    val index: Int,
    val positiveClues: List<ClueValue> = emptyList(),
    val negativeClues: List<ClueValue> = emptyList(),
)
