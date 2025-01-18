package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction

class TeamKnowledgeImpl(
    private val slotsKnowledge: List<List<SlotKnowledge>>,
    private val handsKnowledge: List<HandKnowledge>,
) : TeamKnowledge {
    override fun getSlotKnowledge(playerIndex: Int, slotIndex: Int): SlotKnowledge {
        return slotsKnowledge[playerIndex][slotIndex - 1]
    }

    override fun getAfterDraw(
        drawAction: DrawAction,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        val playerIndex = drawAction.playerMetadata.playerIndex
        val cardsVisibleOnHandsPerPlayer = getCardsVisibleOnHandsPerPlayer()
        val updatedDrawingPlayerSlotsKnowledge = listOf(
            KnowledgeFactory.createSlotKnowledge(
                slotOwnerPlayerMetadata = drawAction.playerMetadata,
                cardsVisibleInHandsPerPlayer = cardsVisibleOnHandsPerPlayer,
                globallyAvailableGameData = globallyAvailableGameData,
            )
        ) + slotsKnowledge[playerIndex]
        val updatedSlotsKnowledge = slotsKnowledge.mapIndexed { index, playerSlotsKnowledge ->
            if (index == playerIndex) updatedDrawingPlayerSlotsKnowledge else playerSlotsKnowledge
        }
        return TeamKnowledgeImpl(
            slotsKnowledge = updatedSlotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    override fun getAfterDrawing(
        drawAction: DrawAction,
        card: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        val playerIndex = drawAction.playerMetadata.playerIndex
        val cardsVisibleOnHandsPerPlayer = getCardsVisibleOnHandsPerPlayer()
        val updatedDrawingPlayerSlotsKnowledge = listOf(
            KnowledgeFactory.createSlotKnowledgeAfterDrawingCard(
                slotOwnerPlayerMetadata = drawAction.playerMetadata,
                cardsVisibleInHandsPerPlayer = cardsVisibleOnHandsPerPlayer,
                globallyAvailableGameData = globallyAvailableGameData,
                visibleCard = card,
            )
        ) + slotsKnowledge[playerIndex].map {
            KnowledgeFactory.createSlotKnowledgeAfterDrawingCard(
                slotOwnerPlayerMetadata = drawAction.playerMetadata,
                cardsVisibleInHandsPerPlayer = cardsVisibleOnHandsPerPlayer,
                globallyAvailableGameData = globallyAvailableGameData,
                visibleCard = card
            )
        }
        val updatedSlotsKnowledge = globallyAvailableGameData.getPlayers().mapIndexed{ index, player ->
            if (index == playerIndex)
                updatedDrawingPlayerSlotsKnowledge
            else {
                slotsKnowledge[index].map { _ ->
                    KnowledgeFactory.createSlotKnowledgeAfterDrawingCard(
                        slotOwnerPlayerMetadata = player.getMetadata(),
                        cardsVisibleInHandsPerPlayer = cardsVisibleOnHandsPerPlayer,
                        globallyAvailableGameData = globallyAvailableGameData,
                        visibleCard = card,
                    )
                }
            }
        }
        return TeamKnowledgeImpl(
            slotsKnowledge = updatedSlotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    override fun getAfterPlay(
        playAction: PlayAction,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        val updatedSlotsKnowledge = slotsKnowledge.mapIndexed { index, playerSlotKnowledge->
            if(playAction.playerMetadata.playerIndex == index) {
                playerSlotKnowledge.toMutableList().removeAt(playAction.slotIndex)
                playerSlotKnowledge
            } else {
                playerSlotKnowledge
                }
        }
        return TeamKnowledgeImpl(
            slotsKnowledge = updatedSlotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    override fun getAfterPlaying(
        playAction: PlayAction,
        playedCard: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfterDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfterDiscarding(
        discardAction: DiscardAction,
        discardedCard: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfterClueGiven(
        clueAction: ClueAction,
        touchedSlotsIndexes: Collection<Int>,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getCardsVisibleOnHands(playerIndex: Int): Collection<HanabiCard> {
        return slotsKnowledge.fold(emptyList()) { acc, slotsKnowledge ->
            acc + slotsKnowledge.fold(emptyList()) { innerAcc, slotKnowledge ->
                val empathy = slotKnowledge.getEmpathy(playerIndex)
                innerAcc + if (empathy.size == 1) listOf(empathy.first()) else emptyList()
            }
        }
    }

    private fun getCardsVisibleOnHandsPerPlayer(): List<Collection<HanabiCard>> {
        return List(slotsKnowledge.size){ index ->
            getCardsVisibleOnHands(index)
        }
    }
}
