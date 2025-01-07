package game.entity.suit

import eelst.ilike.game.entity.Color

object Yellow : BaseClassicSuit(
    id = "yellow",
    name = "yellow",
    abbreviations = listOf("y", "Y", "Yellow"),
) {
    override fun getAssociatedColors(): Collection<Color> {
        return listOf(Color.YELLOW)
    }
}
