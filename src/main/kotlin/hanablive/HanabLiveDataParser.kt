package eelst.ilike.hanablive

import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.engine.player.KnowledgeAwarePlayer
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.SimpleHand
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.entity.dto.instruction.*
import eelst.ilike.hanablive.entity.parsed.CategorizedGameActionData
import eelst.ilike.hanablive.entity.slot.HanabLiveSlot
import eelst.ilike.hanablive.entity.dto.instruction.GameActionType

/**
 * Reads hanab.live produced objects and transforms them into entities understood by the engine
 */
class HanabLiveDataParser(
    private val globallyAvailableGameData: GloballyAvailableGameData,
) {
    fun categorizeGameActionList(actions: List<HanabLiveGameActionData>): CategorizedGameActionData {
        val initialDrawActions = mutableListOf<GameDrawActionData>()
        val actionsByTurn = mutableListOf<MutableList<HanabLiveGameActionData>>(mutableListOf())
        actions.forEach { action->
            if(actionsByTurn.size == 1 && action.type == GameActionType.DRAW){
                initialDrawActions.add(action as GameDrawActionData)
            } else {
                actionsByTurn.last().add(action)
                if (action.type.isTurnDefiningAction) {
                    actionsByTurn.add(mutableListOf())
                }
            }
        }
        return CategorizedGameActionData(
            initialDrawActions = initialDrawActions,
            actionsByTurn = actionsByTurn,
        )
    }

    private fun getInitialDrawActionsGroupedByPlayer(
        actions: List<HanabLiveGameActionData>
    ): Map<PlayerId, List<GameDrawActionData>> {
        val initialDrawActions = actions.takeWhile {
            it.type == GameActionType.DRAW
        }.map { it as GameDrawActionData }
        return initialDrawActions
            .groupBy { it.playerIndex }
            .mapKeys { getPlayerId(it.key) }
    }

    fun parseCard(
        suitIndex: Int,
        rank: Int,
    ): HanabiCard {
        return HanabiCard(
            suit = suitMap[suitIndex]!!,
            rank = rankCluesMap[rank]!!
        )
    }

    fun parseInitialTeamKnowledge(categorizedGameActionData: CategorizedGameActionData): TeamKnowledge {
        val drawActionsGroupedByPlayer = categorizedGameActionData.initialDrawActionsGroupedByPlayerIndex
        val visibleCards = drawActionsGroupedByPlayer.mapValues { entry ->
            entry.value.mapIndexed { index, gameDrawActionData ->
                Pair(index, gameDrawActionData)
            }
                .toMap()
                .filterValues { it.suitIndex >= 0 }
                .mapValues { parseCard(it.value.suitIndex, it.value.rank) }
        }.mapKeys { getPlayerId(it.key) }
        val playersKnowledge = globallyAvailableGameData.playersMetadata.associate {
            it.playerId to KnowledgeFactory
                .createPlayerKnowledge(
                    playerId = it.playerId,
                    globallyAvailableGameData = globallyAvailableGameData,
                    cardsVisibleInPlayerHands = visibleCards,
                    inferredHandKnowledge = globallyAvailableGameData.playersMetadata.associate { playerMetadata ->
                        playerMetadata.playerId to KnowledgeFactory.createEmptyInferredHandKnowledge()
                    }
                )
        }
        return KnowledgeFactory.createTeamKnowledge(playersKnowledge)
    }

    /**
     * This method is an overkill but is just there in case the initial draws don't follow the traditional progressive
     * order
     */
    fun parsePlayers(categorizedGameActionData: CategorizedGameActionData): Map<PlayerId, Player> {
        val drawActionsGroupedByPlayer = getInitialDrawActionsGroupedByPlayer(categorizedGameActionData.initialDrawActions)
        val playersSlots = drawActionsGroupedByPlayer
            .mapValues { drawActions ->
                drawActions.value.map {
                    HanabLiveSlot(
                        index = (it.order % drawActions.value.size) + 1,
                        orderInStartingDeck = it.order
                    )
                }
            }

        return globallyAvailableGameData
            .playersMetadata
            .associate { playerMetadata ->
                playerMetadata.playerId to KnowledgeAwarePlayer(
                    playerId = playerMetadata.playerId,
                    playerIndex = playerMetadata.playerIndex,
                    hand = SimpleHand(slots = playersSlots[playerMetadata.playerId]!!)
                )
            }
    }

    private val suitMap = globallyAvailableGameData.variant.getSuits()
        .mapIndexed { index, suit ->
            Pair(index, suit)
        }.toMap()

    private fun getPlayerId(playerIndex: Int): PlayerId {
        return globallyAvailableGameData.playersMetadata[playerIndex].playerId
    }

    private val availableClueValues = globallyAvailableGameData.variant.getClueValues()
    private val colorClues = availableClueValues.filterIsInstance<Color>()
    private val colorCluesMap = colorClues.mapIndexed { index, color -> Pair(index, color) }.toMap()
    private val rankCluesMap = availableClueValues.filterIsInstance<Rank>().associateBy { it.numericalValue }
}
