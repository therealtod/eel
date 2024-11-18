package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(): PersonalKnowledge {
        return PersonalKnowledgeImpl(
            slotKnowledge = e,
        )
    }

    fun createEmptyPersonalSlotKnowledge(): PersonalSlotKnowledge {
        return PersonalSlotKnowledgeImpl(emptySet())
    }
}