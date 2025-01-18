package eelst.ilike.game.entity.player

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.slot.Slot

class Player(
    private val playerMetadata: PlayerMetadata,
    private val hand: List<Slot>,
) {
    fun getMetadata(): PlayerMetadata {
        return playerMetadata
    }

    fun getSlots(): List<Slot>{
        return hand
    }

    fun getUpdatedAfterDrawing(slot: Slot): Player {
        val updatedHand = listOf(slot) + hand
        return Player(
            playerMetadata = playerMetadata,
            hand = updatedHand
        )
    }

    fun getUpdatedAfterPlaying(slotIndex: Int): Player {
        val updatedHand = hand.minus(hand[slotIndex - 1])
        return Player(
            playerMetadata = playerMetadata,
            hand = updatedHand
        )
    }

    fun getUpdatedAfterDiscarding(slotIndex: Int): Player {
        val updatedHand = hand.minus(hand[slotIndex - 1])
        return Player(
            playerMetadata = playerMetadata,
            hand = updatedHand
        )
    }

    fun getUpdatedAfterClueGiven(clueValue: ClueValue, touchedSlotIndexes: Collection<Int>): Player {
        val updatedHand = hand
            .mapIndexed { index, slot ->
                if (touchedSlotIndexes.contains(index + 1)) {
                    slot.withPositiveClue(clueValue)
                } else {
                    slot.withNegativeClue(clueValue)
                }
            }
        return Player(
            playerMetadata = playerMetadata,
            hand = updatedHand,
        )
    }

    fun forEachSlot(action: (slotIndex: Int, slot: Slot) -> Unit) {
        hand.forEachIndexed { index, slot ->
            action(index + 1, slot)
        }
    }
}