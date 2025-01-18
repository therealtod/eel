package eelst.ilike.engine.knowledge

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerMetadata

object KnowledgeFactory {
    fun createEmptyTeamKnowledge(
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        return createTeamKnowledge(
            slotsKnowledge = List(globallyAvailableGameData.numberOfPlayers) { emptyList() },
            handsKnowledge = List(globallyAvailableGameData.numberOfPlayers) { createEmptyHandKnowledge() },
        )
    }

    fun createTeamKnowledge(
        slotsKnowledge: List<List<SlotKnowledge>>,
        handsKnowledge: List<HandKnowledge>
    ): TeamKnowledge {
        return TeamKnowledgeImpl(
            slotsKnowledge = slotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    fun createEmptyHandKnowledge(): HandKnowledge {
        return HandKnowledgeImpl()
    }

    fun createSlotKnowledge(
        slotOwnerPlayerMetadata: PlayerMetadata,
        cardsVisibleInHandsPerPlayer: List<Collection<HanabiCard>>,
        globallyAvailableGameData: GloballyAvailableGameData
    ): SlotKnowledge {
        val globallyVisibleCards = globallyAvailableGameData.getGloballyVisibleCards()
        val empathyPerPlayer: List<Set<HanabiCard>> by lazy {
            List(cardsVisibleInHandsPerPlayer.size) { index ->
                GameUtils.getSlotEmpathy(
                    visibleCards = globallyVisibleCards + cardsVisibleInHandsPerPlayer[index],
                    positiveClues = emptyList(),
                    negativeClues = emptyList(),
                    suits = globallyAvailableGameData.variant.getSuits(),
                )
            }
        }
        return BaseSlotKnowledge(
            slotOwnerPlayerIndex = slotOwnerPlayerMetadata.playerIndex,
            empathyPerPlayer = empathyPerPlayer,
        )
    }

    fun createSlotKnowledgeAfterDrawingCard(
        slotOwnerPlayerMetadata: PlayerMetadata,
        cardsVisibleInHandsPerPlayer: List<Collection<HanabiCard>>,
        globallyAvailableGameData: GloballyAvailableGameData,
        visibleCard: HanabiCard,
    ): SlotKnowledge {
        val globallyVisibleCards = globallyAvailableGameData.getGloballyVisibleCards()
        val empathyPerPlayer: List<Set<HanabiCard>> by lazy {
            List(cardsVisibleInHandsPerPlayer.size) { index ->
                if (index == slotOwnerPlayerMetadata.playerIndex) GameUtils.getSlotEmpathy(
                    visibleCards = listOf(visibleCard) + globallyVisibleCards + cardsVisibleInHandsPerPlayer[index],
                    positiveClues = emptyList(),
                    negativeClues = emptyList(),
                    suits = globallyAvailableGameData.variant.getSuits(),
                ) else setOf(visibleCard)
            }
        }
        return BaseSlotKnowledge(
            slotOwnerPlayerIndex = slotOwnerPlayerMetadata.playerIndex,
            empathyPerPlayer = empathyPerPlayer,
        )
    }
}
