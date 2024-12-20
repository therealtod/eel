package eelst.ilike.game

import eelst.ilike.game.entity.ClueValue


data class SlotMetadata(
    val index: Int,
    val positiveClues: List<ClueValue>,
    val negativeClues: List<ClueValue>,
) {
    constructor(index: Int): this(index, emptyList(), emptyList())
}
