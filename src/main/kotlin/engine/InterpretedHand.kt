package eelst.ilike.engine

import eelst.ilike.game.Hand
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class InterpretedHand(val slots: Set<InterpretedSlot>): Hand, Set<Slot> by slots {
    override fun holds(card: HanabiCard): Boolean {
        return copiesOf(card) > 0
    }

    override fun holdsAll(cards: Collection<HanabiCard>): Boolean {
        return cards.all { holds(it) }
    }

    override fun getSlot(slotIndex: Int): InterpretedSlot {
        return slots.elementAt(slotIndex - 1)
    }
}
