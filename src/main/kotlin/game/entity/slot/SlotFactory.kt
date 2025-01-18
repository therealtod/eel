package eelst.ilike.game.entity.slot

import eelst.ilike.engine.slot.UnknownSlot

object SlotFactory {
    fun createEmptySlot(): Slot {
        return UnknownSlot(
            positiveClues = emptyList(),
            negativeClues = emptyList(),
        )
    }
}
