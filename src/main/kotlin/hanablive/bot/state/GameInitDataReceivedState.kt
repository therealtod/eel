package eelst.ilike.hanablive.bot.state

import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.*
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.BaseHand
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class GameInitDataReceivedState(
    bot: HanabLiveBot,
    commonState: CommonState,
    private val botPlayerId: PlayerId,
    private val gameInitData: GameInitData,
    private val variantMetadata: VariantMetadata,
    private val game: Game,
): HanabLiveBotState(bot, commonState) {
    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        val botPlayerGloballyAvailableInfo = game.getPlayer(botPlayerId)
        val suitMap = variantMetadata.suits
            .mapIndexed { index, s ->
                Pair(index, game.suits.find { it.name == s }!!)
            }.toMap()
        val rankMap = variantMetadata.clueRanks.associateWith { Rank.getByNumericalValue(it) }
        val actions = gameActionListData.list.filterIsInstance<GameDrawActionData>()
        val playerIndexToIdMap = game.players.values.associate { it.playerIndex to it.playerId }
        val teammateIndexes = game.players
            .minus(botPlayerGloballyAvailableInfo.playerId)
            .map { it.value.playerIndex }
        val drawActionsGroupedByPlayerIndexAndSorted = actions
            .groupBy { it.playerIndex }
            .mapValues { it.value.sortedBy {action -> action.order } }
        val teammatesCards =  teammateIndexes
            .associateWith { teammateIndex->
                drawActionsGroupedByPlayerIndexAndSorted[teammateIndex]!!.map {
                    HanabLiveDataParser.parseCard(
                        draw = it,
                        rankMap = rankMap,
                        suitMap = suitMap,
                    )
                }
            }
        val visibleCardsMap = computeVisibleCardsMap(botPlayerGloballyAvailableInfo, teammatesCards)
        val hands = getHands(
            botPlayerIndex = botPlayerGloballyAvailableInfo.playerIndex,
            teammatesCards = teammatesCards,
            visibleCardsMap = visibleCardsMap,
            playerIndexToIdMap = playerIndexToIdMap,
            suits = game.suits,
        )
        val personalKnowledge = KnowledgeFactory.createEmptyPersonalKnowledge()
        val botPlayer = PlayerFactory.createPlayerPOV(
            playerId = botPlayerGloballyAvailableInfo.playerId,
            game = game,
            personalKnowledge = personalKnowledge,
            playersHands = hands,
        )
        val newState = PlayingState(
            bot = bot,
            commonState = commonState,
            playerPOV = botPlayer,
        )
        bot.state = newState
    }

    private fun getHands(
        botPlayerIndex: Int,
        teammatesCards: Map<Int, List<HanabiCard>>,
        visibleCardsMap: Map<Int, Collection<HanabiCard>>,
        playerIndexToIdMap: Map<Int, PlayerId>,
        suits: Set<Suite>,
    ): Map<PlayerId, Hand> {
        val botPlayerId = playerIndexToIdMap[botPlayerIndex]!!
        val teammatesHands = teammatesCards.mapValues {
            BaseHand(
                ownerId = playerIndexToIdMap[it.key]!!,
                slots = it.value.mapIndexed { index, card->
                    VisibleSlot(
                        globallyAvailableInfo = GloballyAvailableSlotInfo(
                            index = index + 1,
                        ),
                        knowledge = PersonalSlotKnowledgeImpl(
                            ownerId = playerIndexToIdMap[it.key]!!,
                            slotIndex = index + 1,
                            impliedIdentities = emptySet(),
                            empathy = GameUtils.getCardEmpathy(
                                visibleCards = visibleCardsMap[it.key]!!,
                                positiveClues = emptyList(),
                                negativeClues = emptyList(),
                                suits = suits,
                            ),
                        ),
                        visibleCard = card,
                    )
                }.toSet()
            )
        }
        val botSlots = (1..game.defaultHandsSize).map {
            UnknownIdentitySlot(
                globallyAvailableInfo = GloballyAvailableSlotInfo(
                    index = it,
                ),
                knowledge = PersonalSlotKnowledgeImpl(
                    ownerId = playerIndexToIdMap[botPlayerIndex]!!,
                    slotIndex = it,
                    impliedIdentities = emptySet(),
                    empathy = GameUtils.getCardEmpathy(
                        visibleCards = visibleCardsMap[botPlayerIndex]!!,
                        positiveClues = emptyList(),
                        negativeClues = emptyList(),
                        suits = suits,
                    ),
                )
            )
        }
        return teammatesHands.mapKeys { playerIndexToIdMap[it.key]!! } +
                Pair(botPlayerId, BaseHand(botPlayerId, botSlots.toSet()))
    }

    private fun computeVisibleCardsMap(
        botPlayerGloballyAvailableInfo: GloballyAvailablePlayerInfo,
        teammatesCards: Map<Int, List<HanabiCard>>
    ): Map<Int, Collection<HanabiCard>> {
        return teammatesCards.mapValues { item ->
            teammatesCards.minus(item.key).flatMap { it.value }
        } + Pair(botPlayerGloballyAvailableInfo.playerIndex, teammatesCards.flatMap { it.value })
    }
}