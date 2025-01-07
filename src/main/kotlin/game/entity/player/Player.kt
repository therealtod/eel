package eelst.ilike.game.entity.player

import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.slot.Slot

/**
 * Representation of a Hanabi player
 */
interface Player {
    val playerId: PlayerId
    val playerIndex: Int
    val hand: Hand

    fun getSlots(): List<Slot>
    fun getAfterDrawing(slot: Slot): Player
}