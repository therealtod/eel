package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.common.model.metadata.MetadataProviderImpl
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.game.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.model.dto.PlayerPOVDTO
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.TeammateDTO

object InputReader {
    private val mapper = Utils.yamlObjectMapper
    private val metadataProvider = MetadataProviderImpl

    fun getPlayerFromResourceFile(fileName: String): PlayerPOV {
        val fileText = Utils.getResourceFileContentAsString(fileName)
        val dto: ScenarioDTO = mapper.readValue(fileText)
        val variantMetadata = metadataProvider.getVariantMetadata(dto.globallyAvailableInfo.variant)
        val suites = dto.globallyAvailableInfo
            .suites
            .map { metadataProvider.getSuiteMetadata(it) }
            .map { SuiteFactory.createSuite(it, variantMetadata) }
            .toSet()
        val playingStacks = InputParser.parsePlayingStacks(suites, dto.globallyAvailableInfo.playingStacks)
        val trashPile = InputParser.parseTrashPile(dto.globallyAvailableInfo.trashPile, suites)
        val variant = Variant.getVariantByName(dto.globallyAvailableInfo.variant)
        val playersGlobalInfo = dto.globallyAvailableInfo.players.mapIndexed { index, playerDTO ->
            InputParser.parsePlayerGlobalInfo(
                dto = playerDTO,
                playerIndex = index,
                handSize = Utils.getHandSize(dto.globallyAvailableInfo.players.size),
            )
        }
        val activePlayerId = playersGlobalInfo.first().playerId
        val globallyAvailableInfo = GloballyAvailableInfoImpl(
            suits = suites,
            variant = variant,
            players = playersGlobalInfo.associateBy { it.playerId },
            dynamicGloballyAvailableInfo = DynamicGloballyAvailableInfo(
                playingStacks = playingStacks,
                trashPile = trashPile,
                strikes = dto.globallyAvailableInfo.strikes,
                clueTokens = dto.globallyAvailableInfo.clueTokens,
                pace = dto.globallyAvailableInfo.pace,
                efficiency = dto.globallyAvailableInfo.efficiency,
            )
        )
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)
        val playersGlobalInfoMap = playersGlobalInfo.associateBy { it.playerId }

        val visibleCardsMap = computeVisibleCardsMap(
            playerPOV = dto.playerPOV,
            globallyAvailableInfo = globallyAvailableInfo,
            suites = suites,
        )
        val teammatesPersonalSlotKnowledge = dto
            .playerPOV
            .teammates
            .associateBy { it.playerId }
            .mapValues {
                InputParser.parseTeammateSlotKnowledge(
                    teammateDTO = it.value,
                    suites = suites,
                )
            }
        val visibleSlotsKnowledge = dto.playerPOV.teammates
            .associateBy { it.playerId }
            .mapValues { teammate->
                teammate.value.hand.mapIndexed { index, teammateSlotDTO ->
                    KnowledgeFactory.createVisibleSlotKnowledge(
                        visibleCard = InputParser.parseCard(teammate.value.hand[index].card, suites),
                        impliedIdentities = InputParser.parseCards(teammate.value.hand[index].thinks, suites)
                    )
                }
            }
        /*
        val visibleHands = dto.playerPOV.teammates
            .associateBy { it.playerId }
            .mapValues {
                VisibleHand(
                    it.value.hand.mapIndexed { index, slot ->
                        VisibleSlot(
                            globalInfo = playersGlobalInfoMap[it.key]!!.hand
                                .elementAtOrNull(index)
                                ?: GloballyAvailableSlotInfo(
                                    index = index + 1,
                                    positiveClues = emptyList(),
                                    negativeClues = emptyList(),
                                ),
                            card = InputParser.parseCard(slot.card, suites)
                        )
                    }.toSet()
                )
            }

         */
        val activePlayerSlotsKnowledgeAsNotes = (0..<globallyAvailableInfo.defaultHandsSize).map {
            dto.playerPOV.hand.getOrNull(it) ?: "x"
        }
        val activePlayerPersonalKnowledge = InputParser.parseActivePlayerKnowledge(
            knowledgeAsNote = activePlayerSlotsKnowledgeAsNotes,
            suites = suites,
            visibleCards = visibleCardsMap[activePlayerId]!!
        )

        val activePlayerPersonalHandKnowledge = PersonalHandKnowledgeImpl(
            handSize = globallyAvailableInfo.defaultHandsSize,
            slotKnowledge = activePlayerPersonalKnowledge
        )
        val personalHandKnowledge = teammatesPersonalSlotKnowledge.mapValues {
            PersonalHandKnowledgeImpl(
                handSize = globallyAvailableInfo.defaultHandsSize,
                slotKnowledge = it.value
            )
        } + Pair(activePlayerId, activePlayerPersonalHandKnowledge)

        val personalKnowledge = PersonalKnowledgeImpl(
            personalHandKnowledge = personalHandKnowledge,
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
        suites: Set<Suite>,
    ): Map<PlayerId, List<HanabiCard>> {
        val cardsInTrash = globallyAvailableInfo.trashPile.cards
        val cardsInStacks = globallyAvailableInfo.playingStacks.flatMap { it.value.cards }
        val activePlayerKnownCards = playerPOV.hand.filter {
            InputParser.parseCards(it, suites).size == 1
        }.map { InputParser.parseCard(it, suites) }
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
