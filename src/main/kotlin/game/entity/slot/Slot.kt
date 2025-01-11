package eelst.ilike.game.entity.slot

import eelst.ilike.game.entity.ClueValue

/**
 * Represents a Player's slot
 */
interface Slot {
    val index: Int
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>
    fun isTouched(): Boolean
}
