package game.entity.suit

import eelst.ilike.game.entity.Color

object Red : BaseClassicSuit(
    id = "red",
    name = "red",
    abbreviations = listOf("r", "R", "red"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.RED)
    }
}
