package eelst.ilike.game.entity

import eelst.ilike.game.entity.suit.Suit


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
