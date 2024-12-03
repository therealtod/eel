package eelst.ilike.engine.hand

import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

interface InterpretedHand : Hand<InterpretedSlot>{
    fun holds(card: HanabiCard, playerPOV: PlayerPOV): Boolean
    fun copiesOf(card: HanabiCard, playerPOV: PlayerPOV): Int
    fun getSlotsTouchedBy(clueValue: ClueValue, playerPOV: PlayerPOV): Set<Slot>
    fun getSlots(): Set<InterpretedSlot>
    fun isVisibleFrom(playerPOV: PlayerPOV): Boolean
    fun getAsVisible(): VisibleHand
}
