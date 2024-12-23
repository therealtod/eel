package eelst.ilike.engine.player.knowledge

import eelst.ilike.common.exception.UnknownPlayerException
import eelst.ilike.game.PlayerId

class PlayerKnowledgeImpl(
    private val handsKnowledge: Map<PlayerId, HandKnowledge>
): PlayerKnowledge {
    override fun getSlotKnowledge(playerId: PlayerId, slotIndex: Int): SlotKnowledge {
        val handKnowledge = getHandKnowledge(playerId)
        return handKnowledge.getSlotKnowledge(slotIndex)
    }

    override fun getHandKnowledge(playerId: PlayerId): HandKnowledge {
        return handsKnowledge[playerId] ?: throw UnknownPlayerException(
            "No player with playerId: $playerId to retrieve the hand knowledge of"
        )
    }

    override fun getKnowledgeAccessibleTo(playerId: PlayerId): PlayerKnowledge {
        val otherHands = handsKnowledge.minus(playerId)
        return PlayerKnowledgeImpl(
            otherHands + Pair(playerId, getHandKnowledge(playerId).asNotVisible())
        )
    }
}
