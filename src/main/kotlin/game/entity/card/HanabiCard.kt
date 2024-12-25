package eelst.ilike.game.entity.card

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suit

data class HanabiCard(
    val suit: Suit,
    val rank: Rank,
) {
    fun getPrerequisiteCards(): List<HanabiCard> {
        return suit.getCardsBefore(this)
    }

    fun isTouchedBy(clueValue: ClueValue): Boolean {
        return when(clueValue) {
            is Rank -> isTouchedBy(clueValue)
            is Color -> isTouchedBy(clueValue)
            else -> {
                throw UnsupportedOperationException("The given clue value $clueValue has an unsupported type")
            }
        }
    }

    fun isTouchedBy(rank: Rank): Boolean {
        return suit.cluedRankTouches(this.rank, rank)
    }

    fun isTouchedBy(color: Color): Boolean {
        return suit.cluedColorTouches(this.rank, color)
    }
}
