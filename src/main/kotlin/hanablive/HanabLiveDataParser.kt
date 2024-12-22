package eelst.ilike.hanablive

import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.GameData
import eelst.ilike.game.PlayerId
import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.hanablive.model.adapter.GameDataAdapter
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

object HanabLiveDataParser {
    fun parseGloballyAvailableInfo(
        gameInitData: GameInitData,
        variantMetadata: VariantMetadata,
    ): GameData {
        val variant = VariantFactory.createVariant(variantMetadata)
        return GameDataAdapter(
            gameInitData = gameInitData,
            variant = variant,
        )
    }

    fun parseCard(draw: GameDrawActionData, rankMap: Map<Int, Rank>, suitMap: Map<Int, Suite>): HanabiCard {
        return HanabiCard(
            suite = suitMap[draw.suitIndex]!!,
            rank = rankMap[draw.rank]!!
        )
    }

    fun parseCard(
        suitIndex: Int,
        rankIndex: Int, rankMap: Map<Int, Rank>,
        suitMap: Map<Int, Suite>
    ): HanabiCard {
        return HanabiCard(
            suite = suitMap[suitIndex]!!,
            rank = rankMap[rankIndex]!!
        )
    }
    
    fun parseSlot(
        activePlayerId: PlayerId,
        slotOwnerId: PlayerId,
        slotIndex: Int,
        draw: GameDrawActionData,
        indexToSuitMap: Map<Int, Suite>,
        indexToRankMap: Map<Int, Rank>,
        visibleCards: List<HanabiCard>,
        suits: Set<Suite>,
    ): Slot {
        val slotMetadata = SlotMetadata(index = 1)
        val knowledge = KnowledgeFactory.createSlotKnowledge(
            slotOwnerId = slotOwnerId,
            slotIndex = slotIndex,
            impliedIdentities = emptySet(),
            visibleCards = visibleCards,
            positiveClues = emptyList(),
            negativeClues = emptyList(),
            suits = suits,
        )
        if (activePlayerId == slotOwnerId) {
            return UnknownIdentitySlot(
                slotMetadata = slotMetadata,
                knowledge = knowledge
            )
        } else {
            return VisibleSlot(
                slotMetadata = slotMetadata,
                knowledge = knowledge,
                visibleCard = parseCard(
                    draw = draw,
                    rankMap = indexToRankMap,
                    suitMap = indexToSuitMap,
                )
            )
        }
    }
}
