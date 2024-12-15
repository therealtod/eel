package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand


interface PlayerPersonalKnowledge: Knowledge {
    fun canSee(playerId: PlayerId): Boolean
    fun getOwnHandKnowledge(playerId: PlayerId): PlayersHandKnowledge
    fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge
}
