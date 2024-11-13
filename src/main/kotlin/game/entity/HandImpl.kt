package eelst.ilike.game.entity

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.Hand
import eelst.ilike.game.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

/*
class HandImpl(override val slots: Set<Slot>): Hand, Set<Slot> by slots {
    override fun copiesOf(
        card: HanabiCard,
    ): Int {
        return slots.count{ it.getCard() == card }
    }

    override fun holds(
        card: HanabiCard,
        playerPOV: PlayerPOV
    ): Boolean {
        TODO("Not yet implemented")
    }

    override fun holdsAll(
        cards: Collection<HanabiCard>,
        playerPOV: PlayerPOV
    ): Boolean {
        TODO("Not yet implemented")
    }

    override fun getVisibleCards(playerPOV: PlayerPOV): List<HanabiCard> {
        return slots.map { it.getCard()}
    }

    override fun getSlotsTouchedBy(clue: Clue): Set<Slot> {
        return slots.filter { clue.touches(it.getCard()) }.toSet()
    }

    override fun getSlot(slotIndex: Int): Slot {
        return slots.elementAt(slotIndex - 1)
    }
}


 */