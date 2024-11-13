package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.PersonalInfo
import eelst.ilike.game.Hand
import eelst.ilike.game.Slot
import eelst.ilike.game.VisibleSlot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class TeammateHand(slots: Set<VisibleSlot>): InterpretedHand(slots = slots), Set<Slot> by slots {
    override val size = slots.size

    companion object {
        fun from(hand: Hand, personalInfo: PersonalInfo): TeammateHand {
            TODO()
        }
    }

    override fun copiesOf(card: HanabiCard): Int {
        TODO("Not yet implemented")
    }

    override fun getSlot(slotIndex: Int): InterpretedSlot {
        TODO("Not yet implemented")
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        return slots.filter { clue.touches(it.getCard()) }.toSet()
    }

    fun getCards(): List<HanabiCard> {
        return slots.map { it.getCard() }
    }
}