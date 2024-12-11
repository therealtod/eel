package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard


interface PersonalKnowledge {
    fun getImpliedIdentities(slotIndex: Int, playerId: PlayerId): Set<HanabiCard>
    fun getVisibleCard(slotIndex: Int, playerId: PlayerId): HanabiCard
    fun accessibleTo(playerId: PlayerId): PersonalKnowledge
    operator fun plus(knowledge: PersonalKnowledge): PersonalKnowledge
}
