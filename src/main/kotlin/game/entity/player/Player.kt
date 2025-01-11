package eelst.ilike.game.entity.player

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.slot.Slot

/**
 * Representation of a Hanabi player
 */
interface Player {
    val playerId: PlayerId
    val playerIndex: Int
    val hand: Hand

    fun getMetadata(): PlayerMetadata
    fun getAfterDrawing(slot: Slot): Player
    fun getAfterPlaying(slotIndex: Int): Player
    fun getAfterDiscarding(slotIndex: Int): Player
    fun getAfterReceivingClue(clueValue: ClueValue, touchedSlotsIndexes: Set<Int>): Player
}
