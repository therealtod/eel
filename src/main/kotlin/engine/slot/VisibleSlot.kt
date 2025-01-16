package eelst.ilike.engine.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.slot.Slot

class VisibleSlot(
    override val positiveClues: List<ClueValue> = emptyList(),
    override val negativeClues: List<ClueValue> = emptyList(),
    private val identity: HanabiCard,
): Slot {
    override fun isTouched(): Boolean {
        TODO("Not yet implemented")
    }
}
