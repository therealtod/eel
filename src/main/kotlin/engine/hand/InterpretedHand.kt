package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

interface InterpretedHand : Hand<InterpretedSlot> {
    fun holds(card: HanabiCard): Boolean {
        return copiesOf(card) > 0
    }

    fun getKnownSlots(): Set<KnownSlot>
    fun copiesOf(card: HanabiCard): Int
    fun getSlotsTouchedBy(clueValue: ClueValue): Set<Slot>
    fun getSlots(): Set<InterpretedSlot>
}
