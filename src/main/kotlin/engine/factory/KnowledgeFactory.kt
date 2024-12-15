package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(): PlayerPersonalKnowledge {
        return PlayersHandKnowledge()
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