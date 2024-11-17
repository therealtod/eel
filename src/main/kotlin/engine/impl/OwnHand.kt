package eelst.ilike.engine.impl

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

class OwnHand(private val slots: Set<OwnSlot>): InterpretedHand, Set<InterpretedSlot> by slots{
    override fun copiesOf(card: HanabiCard, playerPOV: PlayerPOV): Int {
        return getKnownSlots(playerPOV).count { it.getCard() == card }
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        TODO("Not yet implemented")
    }

    fun getKnownCards(playerPOV: PlayerPOV): List<HanabiCard> {
        return getKnownSlots(playerPOV).map { it.getCard() }
    }

    override fun getKnownSlots(playerPOV: PlayerPOV): Set<InterpretedSlot> {
        return slots.filter { it.isKnown(playerPOV = playerPOV) }.toSet()
    }
}