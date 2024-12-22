package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.GameUtils
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(): PlayerPersonalKnowledge {
        return PlayersHandKnowledge(emptyMap())
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

    fun createSlotKnowledge(
        slotOwnerId: PlayerId,
        slotIndex: Int,
        impliedIdentities: Set<HanabiCard> = emptySet(),
        visibleCards: List<HanabiCard>,
        positiveClues: List<ClueValue> = emptyList(),
        negativeClues: List<ClueValue> = emptyList(),
        suits: Set<Suite>,
    ): PersonalSlotKnowledge {
        val empathy = GameUtils.getCardEmpathy(
            visibleCards = visibleCards,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            suits = suits
        )
        return PersonalSlotKnowledgeImpl(
            ownerId = slotOwnerId,
            slotIndex = slotIndex,
            impliedIdentities = impliedIdentities,
            empathy = empathy,
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