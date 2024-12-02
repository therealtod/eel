package eelst.ilike.game.entity.card

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suite

data class HanabiCard(
    val suite: Suite,
    val rank: Rank,
) {
    fun getPrerequisiteCards(): List<HanabiCard> {
        return suite.getCardsBefore(this)
    }

    fun isTouchedBy(rank: Rank): Boolean {
        return suite.cluedRankTouches(this.rank, rank)
    }

    fun isTouchedBy(color: Color): Boolean {
        return suite.cluedColorTouches(this.rank, color)
    }
}
