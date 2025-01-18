package eelst.ilike.game.entity.slot

import eelst.ilike.game.entity.ClueValue

/**
 * Represents a Player's slot
 */
interface Slot {
    val positiveClues: List<ClueValue>
    val negativeClues: List<ClueValue>

    /**
     * @return true if this slot has been touched by at least one clue
     */
    fun isClued(): Boolean

    /**
     * @return the updated state of this [Slot] after it has been touched by a clue with the given [clueValue]
     */
    fun withPositiveClue(clueValue: ClueValue): Slot

    /**
     * @return the updated state of this [Slot] after a clue with the given [clueValue] has not touched it
     */
    fun withNegativeClue(clueValue: ClueValue): Slot
}
