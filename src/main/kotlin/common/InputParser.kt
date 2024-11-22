package eelst.ilike.utils

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
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
        val suite = Suite.fromAbbreviation(suiteAbbreviation, suites)
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
        suites: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Set<PersonalSlotKnowledge> {
        val slots = knowledge.mapIndexed { index, dto ->
            PersonalSlotKnowledgeImpl(
                impliedIdentities = parseCards(dto, suites),
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
                    suites = suites,
                )
            )
        }
        return slots.toSet()
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
        suites: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): Set<PersonalSlotKnowledge> {
        return parsePlayerSlotKnowledge(
            globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
            knowledge = teammateDTO.hand.map { it.thinks },
            suites = suites,
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
