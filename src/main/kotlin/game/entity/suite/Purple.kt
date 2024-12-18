package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object Purple : BaseClassicSuite(
    id = "purple",
    name = "purple",
    abbreviations = listOf("p", "P", "purple"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.PURPLE)
    }
}
