package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank

sealed class ClassicSuite(
    id: SuiteId,
    name: String,
    abbreviations: Set<Char>,
) : UpwardsSuite(
    id = id,
    name = name,
    abbreviations = abbreviations,
) {
    private val rankTouchMap = mapOf(
        Rank.ONE to setOf(Rank.ONE),
        Rank.TWO to setOf(Rank.TWO),
        Rank.THREE to setOf(Rank.THREE),
        Rank.FOUR to setOf(Rank.FOUR),
        Rank.FIVE to setOf(Rank.FIVE),
    )
    abstract val suiteColors: Set<Color>

    override fun getRanksTouching(rank: Rank): Set<Rank> {
        return rankTouchMap[rank] ?: throw IllegalArgumentException("The given rank: $rank is invalid for this suite")
    }

    override fun getColorsTouching(rank: Rank): Set<Color> {
        return suiteColors
    }
}
