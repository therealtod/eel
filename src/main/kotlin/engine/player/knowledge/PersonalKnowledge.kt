package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.game.PlayerId


interface PersonalKnowledge {
    // fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge
    // fun getSlotKnowledgeAsSet(): Set<PersonalSlotKnowledge>
    fun getVisibleHand(playerId: PlayerId): VisibleHand
    fun getVisibleHands(): Map<PlayerId, VisibleHand>
    fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge
    fun accessibleTo(playerId: PlayerId): PersonalKnowledge
}
