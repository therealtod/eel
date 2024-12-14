package eelst.ilike.engine.hand.slot

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class VisibleHand(
    override val ownerId: PlayerId,
    private val slots: Set<VisibleSlot>
): Hand, Set<Slot> by slots {
    override fun getSlots(): Set<Slot> {
        return slots
    }

    override fun getSlot(slotIndex: Int): Slot {
        return slots.elementAt(slotIndex - 1)
    }

    fun getCardInSlot(slotIndex: Int): HanabiCard {
        return slots.elementAt(slotIndex - 1).knownIdentity
    }

    override fun getSlotsTouchedBy(clueValue: ClueValue): Set<Int> {
        return slots.filter {
            it.knownIdentity.isTouchedBy(clueValue)
        }
            .map { it.index }
            .toSet()
    }

    override fun countCopiesOf(card: HanabiCard): Int {
        return slots.count { it.knownIdentity == card }
    }
}
