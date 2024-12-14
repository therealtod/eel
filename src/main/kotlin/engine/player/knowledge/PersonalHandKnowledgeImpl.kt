package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.game.PlayerId

class PersonalHandKnowledgeImpl(
    private val ownerPlayerId: PlayerId,
    private val handSize: Int,
    private val slotKnowledge: Map<Int, PersonalSlotKnowledge>) : PersonalHandKnowledge {
    override fun getKnowledge(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge[slotIndex] ?: KnowledgeFactory.createEmptyPersonalSlotKnowledge(
            ownerPlayerId = ownerPlayerId,
            slotIndex = slotIndex,
        )
    }

    override fun getHandSize(): Int {
        return handSize
    }
}
