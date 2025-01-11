package eelst.ilike.engine.player

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot

/**
 * A [Player] who takes into account the knowledge acquired by the teammates
 */
class KnowledgeAwarePlayer(
    override val playerId: PlayerId,
    override val playerIndex: Int,
    override val hand: Hand
) : Player {
    override fun getMetadata(): PlayerMetadata {
        return PlayerMetadata(
            playerId = playerId,
            playerIndex = playerIndex,
        )
    }

    override fun getAfterDrawing(slot: Slot): Player {
        TODO("Not yet implemented")
    }

    override fun getAfterPlaying(slotIndex: Int): Player {
        TODO("Not yet implemented")
    }

    override fun getAfterDiscarding(slotIndex: Int): Player {
        TODO("Not yet implemented")
    }

    override fun getAfterReceivingClue(clueValue: ClueValue, touchedSlotsIndexes: Set<Int>): Player {
        TODO("Not yet implemented")
    }

}