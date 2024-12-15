package eelst.ilike.utils

import eelst.ilike.common.model.metadata.MetadataProvider
import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.PlayerPOVDTO
import eelst.ilike.utils.model.dto.SlotDTO

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
        val globallyAvailablePlayerInfo =  dto.playerPOV.players
            .mapIndexed { index, player ->
                GloballyAvailablePlayerInfo(
                    playerId = player.playerId,
                    playerIndex = index,
            ) }
        return GloballyAvailableInfoImpl(
            suits = suites,
            variant = variant,
            players = globallyAvailablePlayerInfo.associateBy { it.playerId },
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

    fun parseCards(text: String, suits: Set<Suite>): Set<HanabiCard> {
        if (text == "x") return emptySet()
        val cardAbbreviations = text.chunked(2)
        return cardAbbreviations.map {
            parseCard(it, suits)
        }.toSet()
    }

    fun parseCard(cardAbbreviation: String, suits: Set<Suite>): HanabiCard {
        val suiteAbbreviation = cardAbbreviation.first()
        val rank = Rank.getByNumericalValue(cardAbbreviation.last().toString().toInt())
        val suite = suits.first { it.abbreviations.contains(suiteAbbreviation.toString()) }
        return HanabiCard(
            suite = suite,
            rank = rank,
        )
    }

    fun parsePlayerSlotKnowledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        playerPOVDTO: PlayerPOVDTO,
        knowledge: List<String>,
        suits: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Map<Int, PersonalSlotKnowledge> {
        val slots = knowledge.mapIndexed { index, dto ->
            index to PersonalSlotKnowledgeImpl(
                ownerId = globallyAvailablePlayerInfo.playerId,
                slotIndex = index + 1,
                impliedIdentities = parseCards(dto, suits),
                empathy = GameUtils.getCardEmpathy(
                    visibleCards = visibleCards,
                    positiveClues = playerPOVDTO.getPlayerDTO(globallyAvailablePlayerInfo.playerId)
                        .hand
                        .elementAt(index)
                        .positiveClues
                        .map { parseClue(it) },
                    negativeClues = playerPOVDTO.getPlayerDTO(globallyAvailablePlayerInfo.playerId)
                        .hand
                        .elementAt(index)
                        .negativeClues
                        .map { parseClue(it) },
                    suits = suits,
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

    fun parsePlayingStacks(suits: Set<Suite>, playingStacksDto: List<List<String>>): Map<SuiteId, PlayingStack> {
        return suits
            .zip(playingStacksDto)
            .associate {
                it.first.id to PlayingStack(
                    suite = it.first,
                    cards = it.second.map { cardAbbreviation -> parseCard(cardAbbreviation, suits) }
                )
            }
    }

    fun parseTrashPile(
        trashCards: List<String>,
        suites: Set<Suite>,
    ): TrashPile {
        return TrashPile(trashCards.map { parseCard(it, suites) })
    }


    fun parseSlot(
        activePlayerId: PlayerId,
        slotOwnerId: PlayerId,
        slotIndex: Int,
        slotDTO: SlotDTO,
        suits: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Slot {
        val globallyAvailableSlotInfo = GloballyAvailableSlotInfo(
            index = slotIndex,
            positiveClues = slotDTO.positiveClues.map { parseClue(it) },
            negativeClues = slotDTO.negativeClues.map { parseClue(it) }
        )
        val knowledge = PersonalSlotKnowledgeImpl(
            ownerId = slotOwnerId,
            slotIndex = slotIndex,
            impliedIdentities = parseCards(slotDTO.thinks, suits),
            empathy = GameUtils.getCardEmpathy(
                visibleCards = visibleCards,
                suits = suits,
                positiveClues = globallyAvailableSlotInfo.positiveClues,
                negativeClues = globallyAvailableSlotInfo.negativeClues
            )
        )

        return if(slotDTO.card != Configuration.UNKNOWN_CARD_SYMBOL) {
            if(activePlayerId == slotOwnerId) {
                FullEmpathySlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                    identity = parseCard(slotDTO.card, suits)
                )
            } else {
                VisibleSlot(
                    globallyAvailableInfo = globallyAvailableSlotInfo,
                    knowledge = knowledge,
                    visibleCard = parseCard(slotDTO.card, suits)
                )
            }
        } else {
            UnknownIdentitySlot(
                globallyAvailableInfo = globallyAvailableSlotInfo,
                knowledge = knowledge,
            )
        }
    }
}
