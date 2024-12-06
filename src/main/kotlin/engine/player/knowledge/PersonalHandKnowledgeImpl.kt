package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.factory.KnowledgeFactory

class PersonalHandKnowledgeImpl(
    private val handSize: Int,
    private val slotKnowledge: Map<Int, PersonalSlotKnowledge>) : PersonalHandKnowledge {
    override fun getKnowledge(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge[slotIndex] ?: KnowledgeFactory.createEmptyPersonalSlotKnowledge()
    }

    override fun getHandSize(): Int {
        return handSize
    }
}
