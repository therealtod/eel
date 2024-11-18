package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.engine.*
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.impl.*
import eelst.ilike.game.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.model.*

object InputReader {
    private val mapper = Common.yamlObjectMapper

    fun parseFile(fileName: String): ActivePlayer {
        val fileText = Common.getResourceFileContentAsString(fileName)
        val dto: BoardStateResource = mapper.readValue(fileText)

        val suites = dto.globallyAvailableInfo.suites.map { Suite.fromId(it) }.toSet()
        val playingStacks = parsePlayingStacks(suites,dto.globallyAvailableInfo.playingStacks)
        val trashPile = TrashPile(dto.globallyAvailableInfo.trashPile.map { CardNoteParser.parseCard(it, suites) })
        val variant = Variant.getVariantByName(dto.globallyAvailableInfo.variant)
        val playersGlobalInfo = dto.globallyAvailableInfo.players.mapIndexed { index, playerDTO->
            parsePlayerGlobalInfo(
                dto = playerDTO,
                playerIndex = index,
                suites = suites
            )
        }
        val activePlayerId = playersGlobalInfo.first().playerId
        val globallyAvailableInfo = GloballyAvailableInfo(
            playingStacks = playingStacks,
            suites = suites,
            trashPile = trashPile,
            strikes = dto.globallyAvailableInfo.strikes,
            efficiency = dto.globallyAvailableInfo.efficiency,
            pace = dto.globallyAvailableInfo.pace,
            score = dto.globallyAvailableInfo.score,
            variant = variant,
            players = playersGlobalInfo.associateBy { it.playerId },
        )
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)
        val activePlayerGloballyAvailableSlotsInfo = activePlayerGloballyAvailableInfo.hand
        val playersGlobalInfoMap = playersGlobalInfo.associateBy { it.playerId }
        val visibleCardsMap: Map<PlayerId, List<HanabiCard>> = computeVisibleCardsMap()
        val cardsSeenByActivePlayer = visibleCardsMap[activePlayerId]!!
        val teammatesPersonalKnowledge = dto
            .playerPOV
            .teammates
            .associateBy { it.playerId }
            .mapValues {
            parsePlayerKnowledge(
                globallyAvailablePlayerInfo = playersGlobalInfoMap[it.key]!!,
                teammateDTO = it.value,
                suites = suites,
                visibleCards = visibleCardsMap[it.key]!!,
            )
        }
        val teammatesHands = dto.playerPOV.teammates
            .associateBy { it.playerId }
            .mapValues {
                TeammateHand(
                    it.value.hand.mapIndexed { index, slot->
                        VisibleSlot(
                            globalInfo = playersGlobalInfoMap[it.key]!!.hand.elementAt(index),
                            card = CardNoteParser.parseCard(slot.card, suites)
                        )
                    }.toSet()
                )
            }
        val personalKnowledge = PersonalKnowledgeImpl(
            slots = dto.playerPOV.hand.mapIndexed { index, slot->
                PersonalSlotKnowledgeImpl(
                    impliedIdentities = CardNoteParser.parseCards(slot, suites),
                    empathy = Utils.getCardEmpathy(
                        visibleCards = cardsSeenByActivePlayer,
                        positiveClues = activePlayerGloballyAvailableSlotsInfo.elementAt(index).positiveClues,
                        negativeClues = activePlayerGloballyAvailableSlotsInfo.elementAt(index).negativeClues,
                        suites = suites,
                    ),
                )
            }.toSet(),
        )


        return PlayerFactory.createActivePlayer(
            playerId = activePlayerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            otherPlayersKnowledge = teammatesPersonalKnowledge,
            teammatesHands = teammatesHands,
        )
    }

    fun parsePlayerGlobalInfo(
        dto: PlayerGloballyAvailableInfoDTO,
        playerIndex: Int,
        suites: Set<Suite>
    ): GloballyAvailablePlayerInfo {
        return GloballyAvailablePlayerInfo(
            playerId = dto.playerId,
            playerIndex = playerIndex,
            hand = dto.slotClues.mapIndexed {index, slotDto->
                GloballyAvailableSlotInfo(
                    index = index + 1,
                    positiveClues = slotDto.positiveClues.map { CardNoteParser.parseClue(it, suites) },
                    negativeClues = slotDto.negativeClues.map { CardNoteParser.parseClue(it, suites) },
            ) }.toSet()
        )
    }

    fun parsePlayerKnowledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        teammateDTO: TeammateDTO,
        suites: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): PersonalKnowledge{

        val slots = teammateDTO.hand.mapIndexed { index, dto ->
            PersonalSlotKnowledgeImpl(
                impliedIdentities = CardNoteParser.parseCards(dto.card, suites),
                empathy = Utils.getCardEmpathy(
                    visibleCards = visibleCards,
                    positiveClues = globallyAvailablePlayerInfo.hand.elementAt(index).positiveClues,
                    negativeClues = globallyAvailablePlayerInfo.hand.elementAt(index).negativeClues,
                    suites = suites,
                )
            )
        }

        return PersonalKnowledgeImpl(
            slots = slots.toSet()
        )
    }

    fun parsePlayingStacks(suites: Set<Suite>, playingStacksDto: List<List<String>>): Map<SuiteId,PlayingStack> {
        return suites
            .zip(playingStacksDto)
            .associate {
                it.first.id to PlayingStack(
                    suite = it.first,
                    cards = it.second.map { cardAbbreviation -> CardNoteParser.parseCard(cardAbbreviation, suites) }
                )
            }
    }

    fun computeVisibleCardsMap(
        playerPOV: PlayerPOVDTO,
        globallyAvailableInfo: GloballyAvailableInfo,
        teammates: Map<PlayerId, TeammateDTO>,
    ): Map<PlayerId, List<HanabiCard>> {
        val cardsInTrash = globallyAvailableInfo.trashPile.cards
        val cardsInStacks = globallyAvailableInfo.playingStacks.flatMap { it.value.cards }
        val activePlayerKnownCards = playerPOV.hand.filter {
            CardNoteParser.
        }
        return globallyAvailableInfo.players.mapValues {
            computeCardsVisibleByPlayer(
                playerId = it.key,
                publiclyVisibleCards = cardsInStacks + cardsInTrash,
                teammates = teammates,
                suites = globallyAvailableInfo.suites,
            )
        }
    }

    fun computeCardsVisibleByPlayer(
        playerId: PlayerId,
        publiclyVisibleCards: List<HanabiCard>,
        teammates: Map<PlayerId, TeammateDTO>,
        suites: Set<Suite>,
    ): List<HanabiCard> {
        val cardInTeammatesHands = teammates
            .filterKeys { it != playerId }
            .flatMap { teammate->
                teammate.value.hand.map {
                    CardNoteParser.parseCard(it.card, suites)
                }
            }

    }
}
