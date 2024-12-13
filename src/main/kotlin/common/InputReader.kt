package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.common.model.metadata.MetadataProviderImpl
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledgeImpl
import eelst.ilike.game.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.utils.model.dto.PlayerPOVDTO
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.TeammateDTO

object InputReader {
    private val mapper = Utils.yamlObjectMapper
    private val metadataProvider = MetadataProviderImpl

    fun getPlayerFromResourceFile(fileName: String): PlayerPOV {
        val fileText = Utils.getResourceFileContentAsString(fileName)
        val dto: ScenarioDTO = mapper.readValue(fileText)
        val globallyAvailableInfo = InputParser.parseGlobalInfo(dto, metadataProvider)
        val activePlayerId = dto.globallyAvailableInfo.players.first().playerId
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)
        val visibleCardsMap = computeVisibleCardsMap(
            playerPOV = dto.playerPOV,
            globallyAvailableInfo = globallyAvailableInfo,
        )
        val teammatesPersonalSlotKnowledge = dto
            .playerPOV
            .teammates
            .associateBy { it.playerId }
            .mapValues {
                InputParser.parseTeammateSlotKnownledge(
                    globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(it.key),
                    teammateDTO = it.value,
                    suits = globallyAvailableInfo.suits,
                    visibleCards = visibleCardsMap[it.key]!!,
                )
            }
        val visibleHands = dto.playerPOV.teammates
            .associateBy { it.playerId }
            .mapValues {
                val slotsInfo = it.value.hand.mapIndexed { index, slot ->
                    globallyAvailableInfo.getPlayerInfo(it.key).hand
                        .elementAtOrNull(index)
                        ?: GloballyAvailableSlotInfo(
                            index = index + 1,
                            positiveClues = emptyList(),
                            negativeClues = emptyList(),
                        )
                }
                val slots = slotsInfo.map { slotInfo ->
                    VisibleSlot(
                        globallyAvailableSlotInfo = slotInfo,
                        visibleCard = InputParser
                            .parseCard(it.value.hand[slotInfo.index - 1].card, globallyAvailableInfo.suits),
                    )
                }
                VisibleHand(
                    ownerId = it.key,
                    slots = slots.toSet(),
                )
            }
        val activePlayerSlotsKnowledge = (0..<globallyAvailableInfo.defaultHandsSize).map {
            dto.playerPOV.hand.getOrNull(it) ?: "x"
        }
        val activePlayerPersonalSlotKnowledge = InputParser.parsePlayerSlotKnowledge(
            globallyAvailablePlayerInfo = activePlayerGloballyAvailableInfo,
            knowledge = activePlayerSlotsKnowledge,
            suits = globallyAvailableInfo.suits,
            visibleCards = visibleCardsMap[activePlayerId]!!
        )

        val activePlayerPersonalHandKnowledge = PersonalHandKnowledgeImpl(
            handSize = globallyAvailableInfo.defaultHandsSize,
            activePlayerPersonalSlotKnowledge,
        )
        val personalHandKnowledge = teammatesPersonalSlotKnowledge.mapValues {
            PersonalHandKnowledgeImpl(
                handSize = globallyAvailableInfo.defaultHandsSize,
                slotKnowledge = it.value
            )
        } + Pair(activePlayerId, activePlayerPersonalHandKnowledge)

        val personalKnowledge = PlayerPersonalKnowledgeImpl(
            personalHandKnowledge = personalHandKnowledge,
            visibleHands = visibleHands
        )

        return PlayerFactory.createPlayerPOV(
            playerId = activePlayerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,

        )
    }

    private fun computeVisibleCardsMap(
        playerPOV: PlayerPOVDTO,
        globallyAvailableInfo: GloballyAvailableInfo,
    ): Map<PlayerId, List<HanabiCard>> {
        val cardsInTrash = globallyAvailableInfo.trashPile.cards
        val cardsInStacks = globallyAvailableInfo.playingStacks.flatMap { it.value.cards }
        val activePlayerKnownCards = playerPOV.hand.filter {
            InputParser.parseCards(it, globallyAvailableInfo.suits).size == 1
        }.map { InputParser.parseCard(it, globallyAvailableInfo.suits) }
        return globallyAvailableInfo.players.mapValues { player ->
            computeCardsVisibleByPlayer(
                playerId = player.key,
                publiclyVisibleCards = cardsInStacks + cardsInTrash + activePlayerKnownCards,
                teammates = playerPOV.teammates.associateBy { it.playerId },
                suits = globallyAvailableInfo.suits,
            )
        }
    }

    private fun computeCardsVisibleByPlayer(
        playerId: PlayerId,
        publiclyVisibleCards: List<HanabiCard>,
        teammates: Map<PlayerId, TeammateDTO>,
        suits: Set<Suite>,
    ): List<HanabiCard> {
        val cardInTeammatesHands = teammates
            .filterKeys { it != playerId }
            .flatMap { teammate ->
                teammate.value.hand.map {
                    InputParser.parseCard(it.card, suits)
                }
            }
        return publiclyVisibleCards + cardInTeammatesHands
    }
}
