package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.game.PlayerId


interface PersonalKnowledge {
    fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge
    fun getSlotKnowledgeAsSet(): Set<PersonalSlotKnowledge>
    fun getTeammateHand(teammatePlayerId: PlayerId): TeammateHand
}