package eelst.ilike.engine.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.slot.Slot

class UnknownSlot(
    override val positiveClues: List<ClueValue> = emptyList(),
    override val negativeClues: List<ClueValue> = emptyList(),
): Slot {
    override fun isTouched(): Boolean {
        TODO("Not yet implemented")
    }
}
