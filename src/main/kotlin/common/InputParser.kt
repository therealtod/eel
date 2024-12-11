package eelst.ilike.utils

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.engine.player.knowledge.VisibleSlotKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.utils.model.dto.PlayerGloballyAvailableInfoDTO
import eelst.ilike.utils.model.dto.TeammateDTO

object InputParser {
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

    fun parseActivePlayerKnowledge(
        knowledgeAsNote: Collection<String>,
        suites: Set<Suite>,
        visibleCards: Collection<HanabiCard>,
    ): Map<Int, PersonalSlotKnowledge> {
        val slots = knowledgeAsNote.mapIndexed { index, dto ->
            index to VisibleSlotKnowledge(
                impliedIdentities = parseCards(dto, suites),
                slotIdentity = visibleCards.elementAt(index),
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

    fun parseTeammateSlotKnowledge(
        teammateDTO: TeammateDTO,
        suites: Set<Suite>,
    ): Map<Int, PersonalSlotKnowledge> {
        return parseActivePlayerKnowledge(
            knowledgeAsNote = teammateDTO.hand.map { it.thinks },
            suites = suites,
            visibleCards = teammateDTO.hand.map { parseCard(it.card, suites) }
        )
    }

    fun parseTrashPile(
        trashCards: List<String>,
        suites: Set<Suite>,
    ): TrashPile {
        return TrashPile(trashCards.map { parseCard(it, suites) })
    }
}
