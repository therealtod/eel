package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class OwnHand(private val slots: Set<OwnSlot>): InterpretedHand, Set<InterpretedSlot> by slots{
    override fun copiesOf(card: HanabiCard): Int {
        return slots.count { it.hasKnownIdentity(card) }
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        TODO("Not yet implemented")
    }

    fun getKnownCards(visibleCards: List<HanabiCard>): List<HanabiCard> {
        return getKnownSlots(visibleCards).map { it.card }
    }

    fun getSlot(slotIndex: Int): OwnSlot {
        return slots.elementAt(slotIndex - 1)
    }

    fun getKnownSlots(visibleCards: List<HanabiCard>): Set<KnownSlot> {
        return slots.filter { it.isKnown(visibleCards = visibleCards) }
            .map {
                KnownSlot(
                    globallyAvailableInfo = it.globalInfo,
                    card = it.getPossibleIdentities(visibleCards).first()
                )
            }.toSet()
    }
}