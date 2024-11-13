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

    fun getRanksTouchingCard(): Set<Rank> {
        return suite.getRanksTouching(rank)
    }

    fun getColorsTouchingCard(): Set<Color> {
        return suite.getColorsTouching(rank)
    }
}
