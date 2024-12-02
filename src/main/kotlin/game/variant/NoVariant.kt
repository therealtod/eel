package eelst.ilike.game.variant

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*


data object NoVariant
    : Variant(
    id = "NVariant",
    name = "No Variant",
    suits = setOf(
        Red,
        Yellow,
        Green,
        Blue,
        Purple,
    )
) {
    override fun getCluableRanks(): Set<Rank> {
        return setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)
    }

    override fun getCluableColors(): Set<Color> {
        return setOf(Color.RED, Color.YELLOW, Color.GREEN, Color.BLUE, Color.PURPLE)
    }
}
