package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suit.Suit
import eelst.ilike.game.entity.suit.SuitId

abstract class BaseClassicSuit(
    id: SuitId,
    name: String,
    abbreviations: List<String>,
) : Suit(
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

    override fun cluedRankTouches(thisSuitRank: Rank, cluedRank: Rank): Boolean {
        return thisSuitRank == cluedRank
    }

    override fun cluedColorTouches(thisSuitRank: Rank, cluedColor: Color): Boolean {
        return getAssociatedColors().contains(cluedColor)
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        return card.rank.numericalValue
    }
}
