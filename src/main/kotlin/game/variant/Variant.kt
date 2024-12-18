package eelst.ilike.game.variant

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suite


abstract class Variant(
    val id: String,
    val name: String,
    val suits: Set<Suite>,
) {
    abstract fun getCluableRanks(): Set<Rank>

    fun getCluableColors(): Set<Color> {
        return suits.flatMap { it.getAssociatedColors() }.toSet()
    }
}
