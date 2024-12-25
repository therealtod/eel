package eelst.ilike.engine.player.knowledge

import eelst.ilike.common.exception.UnknownPlayerException
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class PlayerKnowledgeImpl(
    private val playerId: PlayerId,
    private val globallyVisibleCards: List<HanabiCard>,
    private val handKnowledge: Map<PlayerId, HandKnowledge>,
): PlayerKnowledge {
    override fun getVisibleCards(): List<HanabiCard> {
        return globallyVisibleCards + handKnowledge.flatMap { it.value.getVisibleCards() }
    }

    override fun getOwnHandKnowledge(): HandKnowledge {
        return getHandKnowledge(playerId)
    }

    override fun getTeammateHandKnowledge(playerId: PlayerId): HandKnowledge {
        TODO("Not yet implemented")
    }

    private fun getHandKnowledge(playerId: PlayerId): HandKnowledge {
        return handKnowledge[playerId] ?: throw UnknownPlayerException(
            "No player with playerId: $playerId to retrieve the hand knowledge of"
        )
    }
}