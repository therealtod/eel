package eelst.ilike.utils


import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.engine.*
import eelst.ilike.engine.impl.PersonalInfoImpl
import eelst.ilike.engine.impl.PersonalTeammateInfoImpl
import eelst.ilike.engine.impl.PlayerPOVImpl
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

    fun parseFile(fileName: String): PlayerPOV {
        val fileText = Common.getResourceFileContentAsString(fileName)
        val dto: BoardStateResource = mapper.readValue(fileText)

        val suites = dto.globallyAvailableInfo.suites.map { Suite.fromId(it) }.toSet()
        val playingStacks = suites
            .zip(dto.globallyAvailableInfo.playingStacks)
            .associate {
                it.first.id to PlayingStack(
                    suite = it.first,
                    cards = it.second.map { cardAbbreviation -> parseCard(cardAbbreviation, suites) }
                )
            }
        val trashPile = TrashPile(dto.globallyAvailableInfo.trashPile.map { parseCard(it, suites) })
        val variant = Variant.getVariantByName(dto.globallyAvailableInfo.variant)
        val handSize = Common.getHandSize(dto.globallyAvailableInfo.players.size)
        val players = dto.globallyAvailableInfo.players.mapIndexed { index, playerDTO->
            parsePlayer(
                dto = playerDTO,
                playerIndex = index,
                handSize = handSize,
                suites = suites
            )
        }
        val globallyAvailableInfo = GloballyAvailableInfo(
            playingStacks = playingStacks,
            suites = suites.associateBy { it.id },
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
            parsePlayer(
                teammateDTO = it.value,
                suites = suites,
            )
        }
        val personalInfo = PersonalInfoImpl(
            ownHandInfo = parseOwnHandPersonalInfo(dto.playerPOV.hand, suites) ,
            teammates = personalTeammatesInfo
        )
        return PlayerPOVImpl(
            playerId = dto.playerPOV.playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalInfo = personalInfo
        )
    }

    fun parsePlayer(
        teammateDTO: TeammateDTO,
        suites: Set<Suite>,
    ): PersonalTeammateInfo {
        val slots = teammateDTO.hand.mapIndexed { index, slotDTO->
            getParsedVisibleSlot(
                index = index,
                slotDTO = slotDTO,
                suites = suites,
                playerId = teammateDTO.playerId
            )
        }
        return PersonalTeammateInfoImpl(
            slots = slots.toSet()
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
                impliedIdentities = slotDTO.impliedIdentities.map { parseCard(it, suites) }.toSet()
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
            impliedIdentities = slotDTO.impliedIdentities
                .map { parseCard(cardAbbreviation = it, suites = suites) }
                .toSet()
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

    fun parsePlayer(
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

    fun parseOwnHandPersonalInfo(slots: List<SlotDTO>, suites: Set<Suite>): Set<PersonalSlotInfo> {
        return slots.mapIndexed {index, slot->
            PersonalSlotInfo(
                slotIndex = index + 1,
                impliedIdentities = slot.impliedIdentities.map { parseCard(it, suites) }.toSet()
            )
        }.toSet()
    }
}
