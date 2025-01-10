package eelst.ilike.game.entity.suit

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank

open class ClassicSuit(
    override val id: SuitId,
    override val name: String,
    override val abbreviations: List<String>,
    private val definingColor: Color,
) : BaseSuit (
    abbreviations = abbreviations,
) {
    override fun getAssociatedColors(): List<Color> {
        return listOf(definingColor)
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        return card.rank.numericalValue
    }

    override val suitDirection = SuitDirection.UP

    override val copiesOfMap = mapOf(
        Rank.ONE to 3,
        Rank.TWO to 2,
        Rank.THREE to 2,
        Rank.FOUR to 2,
        Rank.FIVE to 1,
    )

    override fun rankIsTouchedBy(rank: Rank, clueValue: ClueValue): Boolean {
        return when (clueValue) {
            is Color -> clueValue == definingColor
            is Rank -> clueValue == rank
            else -> throw UnsupportedOperationException(
                "The clue value $clueValue of type ${clueValue::class.simpleName} is not supported"
            )
        }
    }

    override fun getLastRank(): Rank {
        return Rank.FIVE
    }

    private val rankTouchMap = mapOf(
        Rank.ONE to setOf(Rank.ONE),
        Rank.TWO to setOf(Rank.TWO),
        Rank.THREE to setOf(Rank.THREE),
        Rank.FOUR to setOf(Rank.FOUR),
        Rank.FIVE to setOf(Rank.FIVE),
    )

    override val ranks = rankTouchMap.keys
}
