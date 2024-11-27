package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.PlayerId

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(): PersonalKnowledge {
        TODO()
    }

    fun createEmptyPersonalSlotKnowledge(): PersonalSlotKnowledge {
        return PersonalSlotKnowledgeImpl(
            impliedIdentities = emptySet(),
            empathy = emptySet(),
        )
    }

    fun createKnowledge(
        playerId: PlayerId,
        slotIndex: Int,
        slotKnowledge: PersonalSlotKnowledge
    ): PersonalKnowledge {
        return PersonalKnowledgeImpl(
            personalHandKnowledge = mapOf(
                playerId to PersonalHandKnowledgeImpl(
                    TODO()
                )
            )
        )
    }
}