package eelst.ilike.engine

import eelst.ilike.game.Slot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

interface InterpretedHand: Set<InterpretedSlot>{
    fun holds(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        return copiesOf(card, playerPOV) > 0
    }

    fun holdsAll(cards: Collection<HanabiCard>, playerPOV: PlayerPOV): Boolean {
        return cards.all { holds(it, playerPOV) }
    }

    fun getKnownSlots(playerPOV: PlayerPOV): Set<InterpretedSlot>
    fun copiesOf(card: HanabiCard,  playerPOV: PlayerPOV): Int
    fun getSlotsTouchedBy(clue: Clue): Set<Slot>
}
