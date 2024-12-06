package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard


interface PersonalKnowledge {
    fun getVisibleHand(playerId: PlayerId): VisibleHand
    fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge
    fun accessibleTo(playerId: PlayerId): PersonalKnowledge
}
