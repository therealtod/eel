package eelst.ilike.hanablive

import eelst.ilike.engine.card.InGameCard
import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.engine.slot.UnknownSlot
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.hanablive.entity.dto.instruction.*
import eelst.ilike.hanablive.entity.parsed.ParsedGameActionList
import eelst.ilike.hanablive.entity.dto.instruction.GameActionType
import engine.card.CardLocationDictionary
import game.exception.UnknownPlayerException

/**
 * Reads hanab.live produced objects and transforms them into entities understood by the engine
 */
class HanabLiveDataParser(variant: Variant, private val playersMetadata: List<PlayerMetadata>) {
    fun parseGameActionList(actions: List<HanabLiveGameActionData>): ParsedGameActionList {
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
        return ParsedGameActionList(
            initialDrawActions = initialDrawActions,
            actionsByTurn = actionsByTurn,
        )
    }

    fun parseDrawAction(drawActionData: GameDrawActionData): DrawAction {
        val playerMetadata = getPlayerMetadata(drawActionData.playerIndex)
        return DrawAction(
            playerMetadata = playerMetadata
        )
    }

    fun parsePlayAction(
        playActionData: GamePlayActionData,
        cardLocationDictionary: CardLocationDictionary,
    ): PlayAction {
        val playerMetadata = getPlayerMetadata(playActionData.playerIndex)
        val slotIndex = cardLocationDictionary.getPlayerSlotIndex(
            playerIndex = playActionData.playerIndex,
            cardOrder = playActionData.order,
        )
        return PlayAction(
            playerMetadata = playerMetadata,
            slotIndex = slotIndex,
        )
    }

    fun parseDiscardAction(
        discardActionData: GameDiscardActionData,
        cardLocationDictionary: CardLocationDictionary,
    ): DiscardAction {
        val playerMetadata = getPlayerMetadata(discardActionData.playerIndex)
        val slotIndex = cardLocationDictionary.getPlayerSlotIndex(
            playerIndex = discardActionData.playerIndex,
            cardOrder = discardActionData.order,
        )
        return DiscardAction(
            playerMetadata = playerMetadata,
            slotIndex = slotIndex,
        )
    }

    fun parseClueAction(
        clueActionData: GameClueActionData,
        cardLocationDictionary: CardLocationDictionary,
    ): ClueAction {
        val clueGiverPlayerMetadata = getPlayerMetadata(clueActionData.giver)
        val clueReceiverPlayerMetadata = getPlayerMetadata(clueActionData.target)
        val clueValue = when(clueActionData.clue.type) {
            0 -> colorCluesMap[clueActionData.clue.value]
            1 -> rankCluesMap[clueActionData.clue.value]
            else -> throw UnsupportedOperationException("Unrecognized clue type: ${clueActionData.clue.type}")
        }
        return ClueAction(
            clueGiver = clueGiverPlayerMetadata,
            clueReceiver = clueGiverPlayerMetadata,
            value = clueValue!!,
        )
    }

    fun parseCardIdentity(
        suitIndex: Int,
        rank: Int,
    ): HanabiCard {
        return HanabiCard(
            suit = suitMap[suitIndex]!!,
            rank = rankCluesMap[rank]!!
        )
    }

    /*
    fun parseInitialTeamKnowledge(parsedGameActionList: ParsedGameActionList): TeamKnowledge {
        val drawActionsGroupedByPlayer = parsedGameActionList.initialDrawActionsGroupedByPlayerIndex
        val visibleCards = drawActionsGroupedByPlayer.mapValues { entry ->
            entry.value.mapIndexed { index, gameDrawActionData ->
                Pair(index, gameDrawActionData)
            }
                .toMap()
                .filterValues { it.suitIndex >= 0 }
                .mapValues { parseCardIdentity(it.value.suitIndex, it.value.rank) }
        }.mapKeys { getPlayerId(it.key) }
        val playersKnowledge = playersMetadata.associate {
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

     */
/*
    /**
     * This method is an overkill but is just there in case the initial draws don't follow the traditional progressive
     * order
     */
    fun parsePlayers(parsedGameActionList: ParsedGameActionList): Map<PlayerId, Player> {
        val drawActionsGroupedByPlayer = getInitialDrawActionsGroupedByPlayer(parsedGameActionList.initialDrawActions)
        val playersSlots = drawActionsGroupedByPlayer
            .mapValues { drawActions ->
                drawActions.value.map {
                    HanabLiveSlot(
                        index = (it.order % drawActions.value.size) + 1,
                        positionInStartingDeck = it.order
                    )
                }
            }

        return playersMetadata
            .associate { playerMetadata ->
                playerMetadata.playerId to KnowledgeAwarePlayer(
                    playerId = playerMetadata.playerId,
                    playerIndex = playerMetadata.playerIndex,
                    hand = SimpleHand(slots = playersSlots[playerMetadata.playerId]!!)
                )
            }
    }

 */

    private val suitMap = variant.getSuits()
        .mapIndexed { index, suit ->
            Pair(index, suit)
        }.toMap()

    private fun getPlayerMetadata(playerIndex: Int): PlayerMetadata {
        return playersMetadataByPlayerIndex[playerIndex]
            ?: throw UnknownPlayerException(
                "Could not find any player metadata corresponding to player index $playerIndex"
            )
    }

    private val availableClueValues = variant.getClueValues()
    private val colorClues = availableClueValues.filterIsInstance<Color>()
    private val colorCluesMap = colorClues.mapIndexed { index, color -> Pair(index, color) }.toMap()
    private val rankCluesMap = availableClueValues.filterIsInstance<Rank>().associateBy { it.numericalValue }
    private val playersMetadataByPlayerIndex = playersMetadata.associateBy { it.playerIndex }
}
