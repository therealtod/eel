package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

interface InterpretedHand: Set<InterpretedSlot>{
    fun holds(card: HanabiCard): Boolean {
        return copiesOf(card) > 0
    }

    fun holdsAll(cards: Collection<HanabiCard>): Boolean {
        return cards.all { holds(it) }
    }

    fun copiesOf(card: HanabiCard): Int
    fun getSlotsTouchedBy(clue: Clue): Set<Slot>
}
