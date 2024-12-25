package eelst.ilike.game.variant

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suit


abstract class Variant(
    val id: String,
    val name: String,
    val suits: Set<Suit>,
) {
    abstract fun getCluableRanks(): Set<Rank>

    fun getCluableColors(): Set<Color> {
        return suits.flatMap { it.getAssociatedColors() }.toSet()
    }
}
