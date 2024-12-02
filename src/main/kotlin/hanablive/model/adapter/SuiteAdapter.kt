package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class SuiteAdapter(private val suiteMetadata: SuiteMetadata, private val variantMetadata: VariantMetadata) : Suite(
    id = suiteMetadata.id,
    name = suiteMetadata.name,
    abbreviations = listOf(suiteMetadata.abbreviation),
    specialRanks = if (variantMetadata.upOrDown) setOf(Rank.START) else emptySet(),
    stackSize = variantMetadata.stackSize
) {
    override fun cluedRankTouches(thisSuiteRank: Rank, cluedRank: Rank): Boolean {
        return !suiteMetadata.noClueRanks && (suiteMetadata.allClueRanks || cluedRank == thisSuiteRank)
    }

    override fun cluedColorTouches(thisSuiteRank: Rank, cluedColor: Color): Boolean {
        return !suiteMetadata.noClueColors
                && (suiteMetadata.allClueColors || suiteMetadata.clueColors.contains(cluedColor.name))
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        TODO()
    }

}
