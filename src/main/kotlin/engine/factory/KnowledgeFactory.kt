package eelst.ilike.engine.factory

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.entity.card.HanabiCard

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

    fun createVisibleSlotKnowledge(
        visibleCard: HanabiCard,
        impliedIdentities: Set<HanabiCard>
    ): PersonalSlotKnowledge {
        return VisibleSlotKnowledge(
            impliedIdentities = impliedIdentities,
            slotIdentity = visibleCard,
        )
    }

    fun createOwnSlotKnowledge(
        impliedIdentities: Set<HanabiCard>,
    ): PersonalSlotKnowledge {
        TODO()
    }
}