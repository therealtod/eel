package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerId
import game.exception.UnknownPlayerException

class PlayerKnowledgeImpl(
    private val playerId: PlayerId,
    private val globallyVisibleCards: List<HanabiCard>,
    private val cardsVisibleInPlayerHands: Map<PlayerId, Map<Int, HanabiCard>>,
    private val inferredHandKnowledge: Map<PlayerId, InferredHandKnowledge>,
) : PlayerKnowledge {
    override fun getVisibleCards(): List<HanabiCard> {
        TODO()
    }

    override fun getVisiblePlayersCards(): Map<PlayerId, Map<Int, HanabiCard>> {
        return cardsVisibleInPlayerHands
    }

    override fun getOwnHandKnowledge(): InferredHandKnowledge {
        return getHandKnowledge(playerId)
    }

    override fun getFullEmpathyCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getKnownCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    private fun getHandKnowledge(playerId: PlayerId): InferredHandKnowledge {
        return inferredHandKnowledge[playerId] ?: throw UnknownPlayerException(
            "No player with playerId: $playerId to retrieve the hand knowledge of"
        )
    }
}
