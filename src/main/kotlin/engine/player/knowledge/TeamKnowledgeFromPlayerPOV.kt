package eelst.ilike.engine.player.knowledge

import eelst.ilike.common.exception.UnknownPlayerException
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class TeamKnowledgeFromPlayerPOV(
    private val povPlayerId: PlayerId,
    private val globallyVisibleCards: List<HanabiCard>,
    private val playersHandsKnowledge: Map<PlayerId, HandKnowledge>
): TeamKnowledge {
    override fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge {
        return PlayerKnowledgeImpl(
            playerId = playerId,
            globallyVisibleCards = globallyVisibleCards,
            handKnowledge = getHandKnowledgeVisibleBy(playerId)
        )
    }

    override fun getAsSeenBy(playerId: PlayerId): TeamKnowledge {
        return TeamKnowledgeFromPlayerPOV(
            povPlayerId = playerId,
            globallyVisibleCards = globallyVisibleCards,
            playersHandsKnowledge = getHandKnowledgeVisibleBy(playerId)
        )
    }

    private fun getHandKnowledgeVisibleBy(playerId: PlayerId): Map<PlayerId, HandKnowledge> {
        val playerHandKnowledge = getHandKnowledge(playerId)
        return playersHandsKnowledge.minus(playerId) + Pair(playerId, playerHandKnowledge.asNotVisible())
    }

    private fun getHandKnowledge(playerId: PlayerId): HandKnowledge {
        return playersHandsKnowledge[playerId] ?: throw UnknownPlayerException(
            "No player with playerId: $playerId to retrieve the hand knowledge of"
        )
    }
}
