package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.model.dto.PlayerPOVDTO
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.TeammateDTO

object InputReader {
    private val mapper = Utils.yamlObjectMapper

    fun getPlayerFromResourceFile(fileName: String): ActivePlayer {
        val fileText = Utils.getResourceFileContentAsString(fileName)
        val dto: ScenarioDTO = mapper.readValue(fileText)
        val suites = dto.globallyAvailableInfo.suites.map { Suite.fromId(it) }.toSet()
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
            playingStacks = playingStacks,
            suites = suites,
            trashPile = trashPile,
            strikes = dto.globallyAvailableInfo.strikes,
            efficiency = dto.globallyAvailableInfo.efficiency,
            pace = dto.globallyAvailableInfo.pace,
            variant = variant,
            players = playersGlobalInfo.associateBy { it.playerId },
            clueTokens = dto.globallyAvailableInfo.clueTokens,
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
                InputParser.parseTeammateSlotKnownledge(
                    globallyAvailablePlayerInfo = playersGlobalInfoMap[it.key]
                        ?: throw IllegalStateException("Player ${it.key} not registered in the game"),
                    teammateDTO = it.value,
                    suites = suites,
                    visibleCards = visibleCardsMap[it.key]!!,
                )
            }
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
        val activePlayerSlotsKnowledge = (0..<globallyAvailableInfo.handsSize).map {
            dto.playerPOV.hand.getOrNull(it) ?: "x"
        }
        val activePlayerPersonalSlotKnowledge = InputParser.parsePlayerSlotKnowledge(
            globallyAvailablePlayerInfo = activePlayerGloballyAvailableInfo,
            knowledge = activePlayerSlotsKnowledge,
            suites = suites,
            visibleCards = visibleCardsMap[activePlayerId]!!
        )

        val activePlayerPersonalHandKnowledge = PersonalHandKnowledgeImpl(activePlayerPersonalSlotKnowledge)
        val personalHandKnowledge = teammatesPersonalSlotKnowledge.mapValues {
            PersonalHandKnowledgeImpl(slotKnowledge = it.value)
        } + Pair(activePlayerId, activePlayerPersonalHandKnowledge)

        val personalKnowledge = PersonalKnowledgeImpl(
            personalHandKnowledge = personalHandKnowledge,
            visibleHands = visibleHands,
        )

        return PlayerFactory.createActivePlayer(
            activePlayerId = activePlayerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )
    }

    private fun computeVisibleCardsMap(
        playerPOV: PlayerPOVDTO,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
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
                suites = globallyAvailableInfo.suites,
            )
        }
    }

    private fun computeCardsVisibleByPlayer(
        playerId: PlayerId,
        publiclyVisibleCards: List<HanabiCard>,
        teammates: Map<PlayerId, TeammateDTO>,
        suites: Set<Suite>,
    ): List<HanabiCard> {
        val cardInTeammatesHands = teammates
            .filterKeys { it != playerId }
            .flatMap { teammate ->
                teammate.value.hand.map {
                    InputParser.parseCard(it.card, suites)
                }
            }
        return publiclyVisibleCards + cardInTeammatesHands
    }
}
