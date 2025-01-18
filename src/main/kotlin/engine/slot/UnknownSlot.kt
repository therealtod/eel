package eelst.ilike.engine.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.slot.Slot

data class UnknownSlot(
    override val positiveClues: List<ClueValue> = emptyList(),
    override val negativeClues: List<ClueValue> = emptyList(),
): Slot {
    override fun isClued(): Boolean {
        return positiveClues.isNotEmpty()
    }

    override fun withPositiveClue(clueValue: ClueValue): Slot {
        return this.copy(positiveClues = listOf(clueValue) + positiveClues)
    }

    override fun withNegativeClue(clueValue: ClueValue): Slot {
        return this.copy(negativeClues = listOf(clueValue) + negativeClues)
    }
}
