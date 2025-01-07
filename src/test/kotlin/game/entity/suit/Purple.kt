package game.entity.suit

import eelst.ilike.game.entity.Color

object Purple : BaseClassicSuit(
    id = "purple",
    name = "purple",
    abbreviations = listOf("p", "P", "purple"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.PURPLE)
    }
}
