package eelst.ilike.hanablive

import eelst.ilike.common.model.metadata.SuitMetadata
import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.Game
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.factory.SuitFactory
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.hanablive.model.adapter.GameAdapter
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

object HanabLiveDataParser {
    fun parseGloballyAvailableInfo(
        gameInitData: GameInitData,
        variantMetadata: VariantMetadata,
        suitsMetadata: Map<String, SuitMetadata>,
    ): Game {
        val suits = suitsMetadata.map { SuitFactory.createSuit(it.value) }
        val variant = VariantFactory.createVariant(variantMetadata, suits.toSet())
        return GameAdapter(
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
}