package eelst.ilike.utils

import eelst.ilike.common.model.metadata.MetadataProvider
import eelst.ilike.engine.factory.SlotFactory
import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.utils.model.dto.ScenarioDTO
import eelst.ilike.utils.model.dto.SlotDTO

object InputParser {
    fun parseGlobalInfo(dto: ScenarioDTO, metadataProvider: MetadataProvider): Game {
        val variantMetadata = metadataProvider.getVariantMetadata(dto.globallyAvailableInfo.variant)
        val suitsMetadata = dto.globallyAvailableInfo
            .suits
            .map { metadataProvider.getSuiteMetadata(it) }
        val suits = suitsMetadata
            .map { SuiteFactory.createSuite(it, variantMetadata) }
            .toSet()
        val playingStacks = parsePlayingStacks(suits, dto.globallyAvailableInfo.playingStacks)
        val trashPile = parseTrashPile(dto.globallyAvailableInfo.trashPile, suits)
        val variant = VariantFactory
            .createVariant(variantMetadata, suits)
        val globallyAvailablePlayerInfo =  dto.playerPOV.players
            .mapIndexed { index, player ->
                GloballyAvailablePlayerInfo(
                    playerId = player.playerId,
                    playerIndex = index,
            ) }
        return GameImpl(
            variant = variant,
            players = globallyAvailablePlayerInfo.associateBy { it.playerId },
            dynamicGloballyAvailableInfo = DynamicGloballyAvailableInfo(
                playingStacks = playingStacks,
                trashPile = trashPile,
                strikes = dto.globallyAvailableInfo.strikes,
                clueTokens = dto.globallyAvailableInfo.clueTokens,
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
        val visibleIdentity = if(slotDTO.card == Configuration.UNKNOWN_CARD_SYMBOL) null
        else parseCard(slotDTO.card, suits)

        return SlotFactory.createSlot(
            activePlayerId = activePlayerId,
            slotOwnerId = slotOwnerId,
            globallyAvailableSlotInfo = globallyAvailableSlotInfo,
            knowledge = knowledge,
            visibleIdentity = visibleIdentity,
        )
    }
}
