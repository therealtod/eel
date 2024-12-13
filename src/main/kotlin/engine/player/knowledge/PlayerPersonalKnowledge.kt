package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.game.PlayerId


interface PlayerPersonalKnowledge: Knowledge {
    fun getVisibleHand(playerId: PlayerId): VisibleHand
    fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge
    fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge
}
