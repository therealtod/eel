package eelst.ilike.hanablive.entity.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.slot.Slot

class HanabLiveSlot(
    override val index: Int,
    /**
     * In the server instructions is named "order".
     *
     * An order of n means that this slot occupied the nth index of the starting deck
     */
    private val orderInStartingDeck: Int,
    override val positiveClues: List<ClueValue> = emptyList(),
    override val negativeClues: List<ClueValue> = emptyList(),
) : Slot {
    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }
}
