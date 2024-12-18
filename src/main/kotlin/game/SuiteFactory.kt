package eelst.ilike.game

import eelst.ilike.common.model.metadata.SuitMetadata
import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteDirection

object SuiteFactory {
    fun createSuite(suiteMetadata: SuitMetadata, variantMetadata: VariantMetadata): Suite {
        return object : Suite(
            id = suiteMetadata.name,
            name = suiteMetadata.name,
            abbreviations = listOf(
                suiteMetadata.abbreviation,
                suiteMetadata.abbreviation.uppercase(),
                suiteMetadata.abbreviation.lowercase(),
                suiteMetadata.name,
                suiteMetadata.name.uppercase(),
                suiteMetadata.name.lowercase(),
            ),
            specialRanks = if (variantMetadata.upOrDown) setOf(Rank.START) else emptySet(),
            stackSize = variantMetadata.stackSize,
            suiteDirection = SuiteDirection.UP
        ) {
            override fun cluedRankTouches(thisSuiteRank: Rank, cluedRank: Rank): Boolean {
                return !suiteMetadata.noClueRanks && (suiteMetadata.allClueRanks || cluedRank == thisSuiteRank)
            }

            override fun cluedColorTouches(thisSuiteRank: Rank, cluedColor: Color): Boolean {
                return !suiteMetadata.noClueColors
                        && (suiteMetadata.allClueColors || suiteMetadata.clueColors.any{ it.equals(cluedColor.name, ignoreCase = true)})
            }

            override fun getPlayingOrder(card: HanabiCard): Int {
                return card.rank.numericalValue
            }

            override fun getAssociatedColors(): Collection<Color> {
                return Color.entries.filter { it.name.equals(name, ignoreCase = true) }
            }
        }
    }
}