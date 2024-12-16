package eelst.ilike.common

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.common.model.metadata.LocalMirrorMetadataProvider
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.SimpleHand
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.utils.Configuration
import eelst.ilike.utils.InputParser
import eelst.ilike.utils.Utils
import eelst.ilike.utils.model.dto.PlayerDTO
import eelst.ilike.utils.model.dto.PlayerPOVDTO
import eelst.ilike.utils.model.dto.ScenarioDTO

object InputReader {
    private val mapper = Utils.yamlObjectMapper
    private val metadataProvider = LocalMirrorMetadataProvider

    fun getPlayerFromResourceFile(fileName: String): ActivePlayer {
        val fileText = Utils.getResourceFileContentAsString(fileName)
        val dto: ScenarioDTO = mapper.readValue(fileText)
        val activePlayerId = dto.playerPOV.playerId
        val globallyAvailableInfo = InputParser.parseGlobalInfo(dto, metadataProvider)
        val playerDTOS = dto.playerPOV.players
        val visibleCardsMap = computeVisibleCardsMap(
            playerPOV = dto.playerPOV,
            globallyAvailableInfo = globallyAvailableInfo,
        )


        val playersSlots = playerDTOS
            .associateBy { it.playerId }
            .mapValues { teammate ->
                teammate.value.hand.mapIndexed { index, slotDTO ->
                    InputParser.parseSlot(
                        activePlayerId = activePlayerId,
                        slotOwnerId = teammate.key,
                        slotIndex = index + 1,
                        slotDTO = slotDTO,
                        suits = globallyAvailableInfo.suits,
                        visibleCards = visibleCardsMap[teammate.key]!!
                    )
                }
            }

        val playersHands = playersSlots.mapValues { SimpleHand(it.key, it.value.toSet()) }

        return PlayerFactory.createPlayerPOV(
            playerId = activePlayerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = KnowledgeFactory.createEmptyPersonalKnowledge(),
            playersHands = playersHands
        )
    }

    private fun computeVisibleCardsMap(
        playerPOV: PlayerPOVDTO,
        globallyAvailableInfo: GloballyAvailableInfo,
    ): Map<PlayerId, List<HanabiCard>> {
        val cardsInTrash = globallyAvailableInfo.trashPile.cards
        val cardsInStacks = globallyAvailableInfo.playingStacks.flatMap { it.value.cards }
        return globallyAvailableInfo.players.mapValues { player ->
            computeCardsVisibleByPlayer(
                playerId = player.key,
                publiclyVisibleCards = cardsInStacks + cardsInTrash,
                teammates = playerPOV.players.associateBy { it.playerId },
                suits = globallyAvailableInfo.suits,
            )
        }
    }

    private fun computeCardsVisibleByPlayer(
        playerId: PlayerId,
        publiclyVisibleCards: List<HanabiCard>,
        teammates: Map<PlayerId, PlayerDTO>,
        suits: Set<Suite>,
    ): List<HanabiCard> {
        val cardInTeammatesHands = teammates
            .filterKeys { it != playerId }
            .flatMap { teammate ->
                teammate.value.hand
                    .filter { it.card != Configuration.UNKNOWN_CARD_SYMBOL }
                    .map {
                    InputParser.parseCard(it.card, suits)
                }
            }
        return publiclyVisibleCards + cardInTeammatesHands
    }
}
