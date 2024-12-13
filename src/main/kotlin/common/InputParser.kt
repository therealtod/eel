package eelst.ilike.utils

import eelst.ilike.common.model.metadata.MetadataProvider
import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.model.dto.PlayerGloballyAvailableInfoDTO
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.TeammateDTO

object InputParser {
    fun parseGlobalInfo(dto: ScenarioDTO, metadataProvider: MetadataProvider): GloballyAvailableInfo {
        val variantMetadata = metadataProvider.getVariantMetadata(dto.globallyAvailableInfo.variant)
        val suites = dto.globallyAvailableInfo
            .suits
            .map { metadataProvider.getSuiteMetadata(it) }
            .map { SuiteFactory.createSuite(it, variantMetadata) }
            .toSet()
        val playingStacks = parsePlayingStacks(suites, dto.globallyAvailableInfo.playingStacks)
        val trashPile = parseTrashPile(dto.globallyAvailableInfo.trashPile, suites)
        val variant = Variant.getVariantByName(dto.globallyAvailableInfo.variant)
        val playersGlobalInfo = dto.globallyAvailableInfo.players.mapIndexed { index, playerDTO ->
            parsePlayerGlobalInfo(
                dto = playerDTO,
                playerIndex = index,
                handSize = Utils.getHandSize(dto.globallyAvailableInfo.players.size),
            )
        }
        return GloballyAvailableInfoImpl(
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
    }

    fun parseCards(text: String, suites: Set<Suite>): Set<HanabiCard> {
        if (text == "x") return emptySet()
        val cardAbbreviations = text.chunked(2)
        return cardAbbreviations.map {
            parseCard(it, suites)
        }.toSet()
    }

    fun parseCard(cardAbbreviation: String, suites: Set<Suite>): HanabiCard {
        val suiteAbbreviation = cardAbbreviation.first()
        val rank = Rank.getByNumericalValue(cardAbbreviation.last().toString().toInt())
        val suite = suites.first { it.abbreviations.contains(suiteAbbreviation.toString()) }
        return HanabiCard(
            suite = suite,
            rank = rank,
        )
    }

    fun parsePlayerGlobalInfo(
        dto: PlayerGloballyAvailableInfoDTO,
        handSize: Int,
        playerIndex: Int,
    ): GloballyAvailablePlayerInfo {
        val slotInfo = (1..handSize).map { index ->
            GloballyAvailableSlotInfo(
                index = index,
                positiveClues = dto.slotClues.getOrNull(index - 1)?.let {
                    it.positiveClues.map { clue ->
                        parseClue(clue)
                    }
                } ?: emptyList(),
                negativeClues = dto.slotClues.getOrNull(index - 1)?.let {
                    it.negativeClues.map { clue ->
                        parseClue(clue)
                    }
                } ?: emptyList(),
            )
        }
        return GloballyAvailablePlayerInfo(
            playerId = dto.playerId,
            playerIndex = playerIndex,
            hand = slotInfo.toSet()
        )
    }

    fun parsePlayerSlotKnowledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        knowledge: List<String>,
        suits: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Map<Int, PersonalSlotKnowledge> {
        val slots = knowledge.mapIndexed { index, dto ->
            index to PersonalSlotKnowledgeImpl(
                impliedIdentities = parseCards(dto, suits),
                empathy = GameUtils.getCardEmpathy(
                    visibleCards = visibleCards,
                    positiveClues = globallyAvailablePlayerInfo
                        .hand
                        .elementAtOrNull(index)
                        ?.positiveClues
                        ?: emptyList(),
                    negativeClues = globallyAvailablePlayerInfo
                        .hand
                        .elementAtOrNull(index)
                        ?.negativeClues
                        ?: emptyList(),
                    suites = suits,
                )
            )
        }
        return slots.associate { it.first + 1 to it.second }
    }

    private fun parseClue(clueAbbreviation: String): ClueValue {
        return Color.entries.find { it.name == clueAbbreviation }
            ?: Rank.entries
                .find { it.numericalValue == clueAbbreviation.toInt() }
            ?: throw IllegalArgumentException("Could not parse clue: $clueAbbreviation")

    }

    fun parsePlayingStacks(suites: Set<Suite>, playingStacksDto: List<List<String>>): Map<SuiteId, PlayingStack> {
        return suites
            .zip(playingStacksDto)
            .associate {
                it.first.id to PlayingStack(
                    suite = it.first,
                    cards = it.second.map { cardAbbreviation -> parseCard(cardAbbreviation, suites) }
                )
            }
    }

    fun parseTeammateSlotKnownledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        teammateDTO: TeammateDTO,
        suits: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Map<Int, PersonalSlotKnowledge> {
        return parsePlayerSlotKnowledge(
            globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
            knowledge = teammateDTO.hand.map { it.thinks },
            suits = suits,
            visibleCards = visibleCards
        )
    }

    fun parseTrashPile(
        trashCards: List<String>,
        suites: Set<Suite>,
    ): TrashPile {
        return TrashPile(trashCards.map { parseCard(it, suites) })
    }
}
