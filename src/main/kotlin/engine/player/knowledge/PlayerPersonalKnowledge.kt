package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand


interface PlayerPersonalKnowledge: Knowledge {
    fun canSee(playerId: PlayerId): Boolean
    fun getVisibleHand(playerId: PlayerId): VisibleHand
    fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge
    fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge
}
