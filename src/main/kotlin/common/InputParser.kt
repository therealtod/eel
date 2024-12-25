package eelst.ilike.utils

import common.model.dto.PlayerDTO
import eelst.ilike.common.model.metadata.MetadataProvider
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suit
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.utils.model.dto.ScenarioDTO
import common.model.dto.SlotDTO
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.knowledge.HandKnowledge
import eelst.ilike.engine.player.knowledge.HandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.SlotKnowledge

object InputParser {
    fun parseGlobalInfo(dto: ScenarioDTO, metadataProvider: MetadataProvider): GameData {
        val variantMetadata = metadataProvider.getVariantMetadata(dto.globallyAvailableInfo.variant)
        val variant = VariantFactory
            .createVariant(variantMetadata)
        val playingStacks = parsePlayingStacks(variant.suits, dto.globallyAvailableInfo.playingStacks)
        val trashPile = parseTrashPile(dto.globallyAvailableInfo.trashPile, variant.suits)
        val playerMetadata =  dto.playerPOV.players
            .mapIndexed { index, player ->
                PlayerMetadata(
                    playerId = player.playerId,
                    playerIndex = index,
            ) }
        return GameDataImpl(
            variant = variant,
            players = playerMetadata.associateBy { it.playerId },
            dynamicGameData = DynamicGameData(
                playingStacks = playingStacks,
                trashPile = trashPile,
                strikes = dto.globallyAvailableInfo.strikes,
                clueTokens = dto.globallyAvailableInfo.clueTokens,
            )
        )
    }

    fun parseCards(text: String, suits: Set<Suit>): Set<HanabiCard> {
        if (text == "x") return emptySet()
        val cardAbbreviations = text.chunked(2)
        return cardAbbreviations.map {
            parseCard(it, suits)
        }.toSet()
    }

    fun parseCard(cardAbbreviation: String, suits: Set<Suit>): HanabiCard {
        val suiteAbbreviation = cardAbbreviation.first()
        val rank = Rank.getByNumericalValue(cardAbbreviation.last().toString().toInt())
        val suite = suits.first { it.abbreviations.contains(suiteAbbreviation.toString()) }
        return HanabiCard(
            suit = suite,
            rank = rank,
        )
    }

    private fun parseClue(clueAbbreviation: String): ClueValue {
        return Color.entries.find { it.name == clueAbbreviation }
            ?: Rank.entries
                .find { it.numericalValue == clueAbbreviation.toInt() }
            ?: throw IllegalArgumentException("Could not parse clue: $clueAbbreviation")

    }

    fun parsePlayingStacks(suits: Set<Suit>, playingStacksDto: List<List<String>>): Map<SuiteId, PlayingStack> {
        return suits
            .zip(playingStacksDto)
            .associate {
                it.first.id to PlayingStack(
                    suit = it.first,
                    cards = it.second.map { cardAbbreviation -> parseCard(cardAbbreviation, suits) }
                )
            }
    }

    fun parseTrashPile(
        trashCards: List<String>,
        suits: Set<Suit>,
    ): TrashPile {
        return TrashPile(trashCards.map { parseCard(it, suits) })
    }

    fun parseSlotGlobalInfo(playerDTO: PlayerDTO): List <SlotMetadata> {
        return playerDTO.hand.mapIndexed { index, slotDTO ->
            SlotMetadata(
                index = index + 1,
                positiveClues = slotDTO.positiveClues.map { parseClue(it) },
                negativeClues = slotDTO.negativeClues.map { parseClue(it) }
            )
        }
    }

    fun parseHandKnowledge(playerDTO: PlayerDTO, suits: Set<Suit>): HandKnowledge {
        val slotKnowledge = playerDTO.hand.mapIndexed { index, slotDTO ->
            Pair(index + 1, parseSlotKnowledge(slotDTO, suits))
        }.toMap()
        return HandKnowledgeImpl(slotKnowledge.toMutableMap())
    }

    fun parseSlotKnowledge(slotDTO: SlotDTO, suits: Set<Suit>): SlotKnowledge {
        val card = if (slotDTO.card == Configuration.UNKNOWN_CARD_SYMBOL) {
            null
        } else {
            parseCard(
                cardAbbreviation = slotDTO.card,
                suits = suits,
            )
        }
        return KnowledgeFactory.createSlotKnowledge(
            visibleCard = card,
            signals = emptyMap(), //TODO: implement
            impliedIdentities = parseCards(slotDTO.thinks, suits),
            hasConflictingInformation = false //TODO: implement
        )
    }
}
