package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

sealed class ClassicSuite(
    id: SuiteId,
    name: String,
    abbreviations: List<String>,
) : Suite(
    id = id,
    name = name,
    abbreviations = abbreviations,
    stackSize = 5,
) {
    private val rankTouchMap = mapOf(
        Rank.ONE to setOf(Rank.ONE),
        Rank.TWO to setOf(Rank.TWO),
        Rank.THREE to setOf(Rank.THREE),
        Rank.FOUR to setOf(Rank.FOUR),
        Rank.FIVE to setOf(Rank.FIVE),
    )
    abstract val suiteColors: Set<Color>

    override fun cluedRankTouches(thisSuiteRank: Rank, cluedRank: Rank): Boolean {
        return thisSuiteRank == cluedRank
    }

    override fun cluedColorTouches(thisSuiteRank: Rank, cluedColor: Color): Boolean {
        return suiteColors.contains(cluedColor)
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        return card.rank.numericalValue
    }
}
