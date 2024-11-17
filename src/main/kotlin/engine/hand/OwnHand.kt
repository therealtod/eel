package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class OwnHand(private val slots: Set<OwnSlot>) : InterpretedHand, Set<InterpretedSlot> by slots {
    override fun copiesOf(card: HanabiCard): Int {
        return slots.count { it.hasKnownIdentity(card) }
    }

    override fun getSlotsTouchedBy(clueValue: ClueValue): Set<Slot> {
        TODO("Not yet implemented")
    }

    fun getKnownCards(): List<HanabiCard> {
        return getKnownSlots().map { it.card }
    }

    override fun getSlot(slotIndex: Int): OwnSlot {
        return slots.elementAt(slotIndex - 1)
    }

    override fun getSlots(): Set<InterpretedSlot> {
        return slots
    }

    override fun getKnownSlots(): Set<KnownSlot> {
        return slots.filter { it.isKnown() }
            .map {
                KnownSlot(
                    globallyAvailableInfo = it.globalInfo,
                    card = it.getPossibleIdentities().first()
                )
            }.toSet()
    }
}
