package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

interface InterpretedHand : Hand {
    fun holds(card: HanabiCard): Boolean {
        return copiesOf(card) > 0
    }
    fun getKnownSlots(): Set<KnownSlot>
    fun copiesOf(card: HanabiCard): Int
    fun getSlotsTouchedBy(clue: Clue): Set<Slot>
    fun getSlots(): Set<InterpretedSlot>
}
