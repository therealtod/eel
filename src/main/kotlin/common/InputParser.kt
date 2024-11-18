package eelst.ilike.utils

import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.action.Clue
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.TrashPile
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
        suites: Set<Suite>
    ): GloballyAvailablePlayerInfo {
        val slotInfo = (1..handSize).mapIndexed { index, i ->
            GloballyAvailableSlotInfo(
                index = index + 1,
                positiveClues = dto.slotClues.getOrNull(index)?.let {
                    it.positiveClues.map { clue ->
                        parseClue(dto.playerId, clue, suites)
                    }
                } ?: emptyList(),
                negativeClues = dto.slotClues.getOrNull(index)?.let {
                    it.negativeClues.map { clue ->
                        parseClue(dto.playerId, clue, suites)
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

    fun parsePlayerKnowledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        knowledge: List<String>,
        suites: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): PersonalKnowledge {

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

        return PersonalKnowledgeImpl(
            slots = slots.toSet()
        )
    }

    fun parseClue(playerId: PlayerId, clueAbbreviation: String, suites: Set<Suite>): Clue {
        return Color.entries.find { it.name == clueAbbreviation }
            ?.let {
                ColorClue(
                    color = it,
                    receiver = playerId
                )
            } ?: Rank.entries
            .find { it.numericalValue == clueAbbreviation.toInt() }
            ?.let {
                RankClue(
                    rank = it,
                    receiver = playerId,
                )
            } ?: throw IllegalArgumentException("Could not parse clue: $clueAbbreviation")

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

    fun parseTeammateKnownledge(
        globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
        teammateDTO: TeammateDTO,
        suites: Set<Suite>,
        visibleCards: List<HanabiCard>,
    ): PersonalKnowledge {

        return parsePlayerKnowledge(
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
