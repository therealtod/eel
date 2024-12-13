package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

interface Hand: Set<Slot> {
    val ownerId: PlayerId

    fun getSlots(): Set<Slot>
    fun getSlot(slotIndex: Int): Slot
    fun getSlotsTouchedBy(clueValue: ClueValue): Set<Int>
    fun countCopiesOf(card: HanabiCard): Int
}
