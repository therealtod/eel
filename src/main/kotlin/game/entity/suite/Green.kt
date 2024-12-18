package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object Green : BaseClassicSuite(
    id = "green",
    name = "green",
    abbreviations = listOf("g", "G", "green"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.GREEN)
    }
}
