package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object Yellow : BaseClassicSuite(
    id = "yellow",
    name = "yellow",
    abbreviations = listOf("y", "Y", "Yellow"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.YELLOW)
    }
}
