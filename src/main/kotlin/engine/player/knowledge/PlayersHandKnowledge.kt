package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId

class PlayersHandKnowledge(
    private val knowledge: Map<PlayerId, Any>
): PlayerPersonalKnowledge {
    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO("Not yet implemented")
    }

    override fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge {
        return PlayersHandKnowledge(knowledge.minus(playerId))
    }

    override fun canSee(playerId: PlayerId): Boolean {
        TODO("Not yet implemented")
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PlayersHandKnowledge {
        return this
    }
}
