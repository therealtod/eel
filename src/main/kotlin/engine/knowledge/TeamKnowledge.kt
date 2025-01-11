package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.player.PlayerId

interface TeamKnowledge {
    fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge
}
