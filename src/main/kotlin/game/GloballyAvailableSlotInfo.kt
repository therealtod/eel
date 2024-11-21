package eelst.ilike.game

import eelst.ilike.game.entity.clue.Clue


data class GloballyAvailableSlotInfo(
    val index: Int,
    val positiveClues: List<Clue>,
    val negativeClues: List<Clue>,
)
