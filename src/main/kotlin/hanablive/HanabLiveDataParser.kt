package eelst.ilike.hanablive

import eelst.ilike.game.GameConstants
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.slot.Slot
import eelst.ilike.game.entity.slot.SlotMetadata
import eelst.ilike.game.entity.suit.Suit
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.hanablive.entity.dto.instruction.GameDrawActionData
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData

/**
 * Reads hanab.live produces objects and transforms them into entities understood by the engine
 */
object HanabLiveDataParser {
    fun parseGloballyAvailableInfo(
        gameInitData: GameInitData,
        variantMetadata: VariantMetadata,
        suitsMetadata: Map<SuitId, SuitMetadata>
    ): GloballyAvailableGameData {
        val variant = VariantFactory.createVariant(variantMetadata, suitsMetadata)
        val suits = variant.getSuits()
        return GloballyAvailableGameData(
            variant = variant,
            playingStacks = suits.associate{ it.id to PlayingStack(emptyList(), it) },
            trashPile = TrashPile(),
            strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
            clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
            numberOfPlayers = gameInitData.playerNames.size,
            amountOfCardsPlayed = 0,
            possibleMaxScore = variantMetadata.stackSize * suits.size ,
        )
    }

    fun parseCard(draw: GameDrawActionData, rankMap: Map<Int, Rank>, suitMap: Map<Int, Suit>): HanabiCard {
        return HanabiCard(
            suit = suitMap[draw.suitIndex]!!,
            rank = rankMap[draw.rank]!!
        )
    }

    fun parseCard(
        suitIndex: Int,
        rankIndex: Int, rankMap: Map<Int, Rank>,
        suitMap: Map<Int, Suit>
    ): HanabiCard {
        return HanabiCard(
            suit = suitMap[suitIndex]!!,
            rank = rankMap[rankIndex]!!
        )
    }

    fun parseSlot(
        activePlayerId: PlayerId,
        slotOwnerId: PlayerId,
        slotIndex: Int,
        draw: GameDrawActionData,
        indexToSuitMap: Map<Int, Suit>,
        indexToRankMap: Map<Int, Rank>,
        visibleCards: List<HanabiCard>,
        suits: Set<Suit>,
    ): Slot {
        val slotMetadata = SlotMetadata(index = 1)
        /*
        val knowledge = KnowledgeFactory.createSlotKnowledge(
            slotOwnerId = slotOwnerId,
            slotIndex = slotIndex,
            impliedIdentities = emptySet(),
            visibleCards = visibleCards,
            positiveClues = emptyList(),
            negativeClues = emptyList(),
            suits = suits,
        )

         */
        /*
        if (activePlayerId == slotOwnerId) {
            return UnknownIdentitySlot(
                slotMetadata = slotMetadata,
                knowledge = TODO()
            )
        } else {
            return VisibleSlot(
                slotMetadata = slotMetadata,
                knowledge = TODO(),
                visibleCard = parseCard(
                    draw = draw,
                    rankMap = indexToRankMap,
                    suitMap = indexToSuitMap,
                )
            )
        }

         */
        TODO()
    }
}
