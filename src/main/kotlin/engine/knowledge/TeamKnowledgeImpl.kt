package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.player.PlayerId
import game.exception.UnknownPlayerException

class TeamKnowledgeImpl(private val playersKnowledge: Map<PlayerId, PlayerKnowledge>) : TeamKnowledge {
    override fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge {
        return playersKnowledge[playerId]
            ?: throw UnknownPlayerException("Cannot find knowledge information for player with id: $playerId")
    }
}
