package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class OwnHand(slots: Set<OwnSlot>, private val playerPOV: PlayerPOV): InterpretedHand(slots){
    override fun copiesOf(card: HanabiCard): Int {
        return getKnownSlots().count { it.getCard() == card }
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        TODO("Not yet implemented")
    }

    fun getKnownCards(): List<HanabiCard> {
        return getKnownSlots().map { it.getCard() }
    }

    fun getKnownSlots(): Set<InterpretedSlot> {
        return slots.filter { it.isKnown(playerPOV = playerPOV) }.toSet()
    }
}