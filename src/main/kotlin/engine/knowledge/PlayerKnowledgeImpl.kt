package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerId
import game.exception.UnknownPlayerException

class PlayerKnowledgeImpl(
    private val playerId: PlayerId,
    private val globallyVisibleCards: List<HanabiCard>,
    private val handKnowledge: Map<PlayerId, HandKnowledge>,
): PlayerKnowledge {
    override fun getVisibleCards(): List<HanabiCard> {
        TODO()
    }

    override fun getOwnHandKnowledge(): HandKnowledge {
        return getHandKnowledge(playerId)
    }

    override fun getFullEmpathyCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getKnownCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    private fun getHandKnowledge(playerId: PlayerId): HandKnowledge {
        return handKnowledge[playerId] ?: throw UnknownPlayerException(
            "No player with playerId: $playerId to retrieve the hand knowledge of"
        )
    }
}
