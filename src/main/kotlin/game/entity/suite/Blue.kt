package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object Blue : BaseClassicSuite(
    id = "blue",
    name = "blue",
    abbreviations = listOf("b", "B", "blue"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.BLUE)
    }
}
