package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.engine.*
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.impl.PersonalInfoImpl
import eelst.ilike.engine.impl.ActivePlayer
import eelst.ilike.engine.impl.InfoOnTeammateImpl
import eelst.ilike.game.*
import eelst.ilike.game.action.Clue
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
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
        val trashPile = TrashPile(dto.globallyAvailableInfo.trashPile.map { parseCard(it, suites) })
        val variant = Variant.getVariantByName(dto.globallyAvailableInfo.variant)
        val handSize = Common.getHandSize(dto.globallyAvailableInfo.players.size)
        val players = dto.globallyAvailableInfo.players.mapIndexed { index, playerDTO->
            parseTeammateInfo(
                dto = playerDTO,
                playerIndex = index,
                handSize = handSize,
                suites = suites
            )
        }
        val globallyAvailableInfo = GloballyAvailableInfo(
            playingStacks = playingStacks,
            suites = suites,
            trashPile = trashPile,
            strikes = dto.globallyAvailableInfo.strikes,
            efficiency = dto.globallyAvailableInfo.efficiency,
            pace = dto.globallyAvailableInfo.pace,
            score = dto.globallyAvailableInfo.score,
            variant = variant,
            players = players.associateBy { it.playerId },
        )
        val personalTeammatesInfo = dto
            .playerPOV
            .teammates
            .associateBy { it.playerId }
            .mapValues {
            parseTeammateInfo(
                teammateDTO = it.value,
                suites = suites,
            )
        }
        val personalInfo = PersonalInfoImpl(
            ownHandInfo = parseOwnHandPersonalInfo(dto.playerPOV.hand, handSize, suites),
            teammates = personalTeammatesInfo
        )
        return PlayerFactory.createActivePlayer(
            playerId = players.first().playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalInfo = personalInfo
        )
    }

    fun parseTeammateInfo(
        teammateDTO: TeammateDTO,
        suites: Set<Suite>,
    ): InfoOnTeammate{

        val slotInfo = teammateDTO.hand.mapIndexed { index, slotDTO->
            getParsedVisibleSlot(
                index = index,
                slotDTO = slotDTO,
                suites = suites,
                playerId = teammateDTO.playerId
            )
        }

        return InfoOnTeammateImpl(
            slots = slotInfo.toSet(),
            teammatePOVInfo = emptySet()
        )
    }

    fun parseSlots(slotDTOs: List<SlotDTO>, suites: Set<Suite>, playerId: PlayerId): Set<Slot> {
        return slotDTOs.mapIndexed { index, dto ->
            parseSlot(index, dto, suites, playerId)
        }.toSet()
    }

    fun parseSlot(index: Int, slotDTO: SlotDTO, suites: Set<Suite>, playerId: PlayerId): Slot {
        return when (slotDTO.cardAbbreviation) {
            "x" -> OwnSlot(
                globalInfo = GloballyAvailableSlotInfo(
                    index = index + 1,
                    positiveClues = parseSlotClues(slotDTO.positiveClues, playerId),
                    negativeClues = parseSlotClues(slotDTO.negativeClues, playerId),
                ),
                impliedIdentities = slotDTO.impliedIdentities.map { parseCard(it, suites) }.toSet(),
                suites = suites
            )

            else -> getParsedVisibleSlot(index, slotDTO, suites, playerId)
        }
    }

    fun parseSlotClues(clues: List<String>, playerId: PlayerId): List<Clue> {
        return clues.map { clueAbbreviation ->
            Color.entries
                .firstOrNull { it.name == clueAbbreviation }
                ?.let {
                    ColorClue(
                        color = it,
                        receiver = playerId
                    )
                }
                ?: Rank.entries
                    .firstOrNull { it.numericalValue == clueAbbreviation.toIntOrNull() }
                    ?.let {
                        RankClue(
                            rank = it,
                            receiver = playerId
                        )
                    }
                ?: throw kotlin.IllegalArgumentException("Clue $clueAbbreviation cannot be parsed")
        }
    }

    fun getParsedVisibleSlot(index: Int, slotDTO: SlotDTO, suites: Set<Suite>, playerId: PlayerId): VisibleSlot {
        return VisibleSlot(
            globalInfo = GloballyAvailableSlotInfo(
                index = index + 1,
                positiveClues = parseSlotClues(slotDTO.positiveClues, playerId),
                negativeClues = parseSlotClues(slotDTO.negativeClues, playerId)
            ),
            card = parseCard(
                cardAbbreviation = slotDTO.cardAbbreviation,
                suites = suites,
            ),
        )
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

    fun parseTeammateInfo(
        dto: PlayerGloballyAvailableInfoDTO,
        playerIndex: Int,
        handSize: Int,
        suites: Set<Suite>
    ): GloballyAvailablePlayerInfo {
        val slots = dto.slots.ifEmpty {
            List(size = handSize) { SlotDTO(
                cardAbbreviation = "x"
            ) }
        }.map {
            parseSlot(
                index = playerIndex,
                slotDTO = it,
                suites = suites,
                playerId = dto.playerId
            )
        }
        return GloballyAvailablePlayerInfo(
            playerId = dto.playerId,
            playerIndex = playerIndex,
            hand = slots.mapIndexed { index, info->
                GloballyAvailableSlotInfo(
                index = index + 1,
                    positiveClues = info.positiveClues,
                    negativeClues = info.negativeClues
            ) }.toSet()
        )
    }

    fun parseOwnHandPersonalInfo(slots: List<SlotDTO>, handSize: Int, suites: Set<Suite>): Set<PersonalSlotInfo> {
        return slots
            .ifEmpty { List(handSize) { SlotDTO("x") } }
            .mapIndexed {index, slot->
            PersonalSlotInfo(
                slotIndex = index + 1,
                impliedIdentities = slot.impliedIdentities.map { parseCard(it, suites) }.toSet()
            )
        }.toSet()
    }

    fun parsePlayingStacks(suites: Set<Suite>, playingStacksDto: List<List<String>>): Map<SuiteId,PlayingStack> {
        return suites
            .zip(playingStacksDto)
            .associate {
                it.first.id to PlayingStack(
                    suite = it.first,
                    cards = it.second.map { cardAbbreviation -> parseCard(cardAbbreviation, suites) }
                )
            }
    }
}
