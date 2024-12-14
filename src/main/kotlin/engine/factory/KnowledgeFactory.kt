package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(): PlayerPersonalKnowledge {
        TODO()
    }

    fun createEmptyPersonalSlotKnowledge(
        ownerPlayerId: PlayerId,
        slotIndex: Int,
    ): PersonalSlotKnowledge {
        return PersonalSlotKnowledgeImpl(
            ownerId = ownerPlayerId,
            slotIndex = slotIndex,
            impliedIdentities = emptySet(),
            empathy = emptySet(),
        )
    }

    fun createKnowledge(
        playerId: PlayerId,
        slotIndex: Int,
        possibleIdentities: Set<HanabiCard>,
        empathy: Set<HanabiCard>,
    ): Knowledge {
        return PersonalSlotKnowledgeImpl(
            ownerId = playerId,
            slotIndex = slotIndex,
            impliedIdentities = possibleIdentities,
            empathy = empathy,
        )
    }
}