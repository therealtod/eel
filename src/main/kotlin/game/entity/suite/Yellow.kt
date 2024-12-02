package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Yellow : ClassicSuite(
    id = "yellow",
    name = "yellow",
    abbreviations = listOf("y"),
) {
    override val suiteColors = setOf(Color.YELLOW)
}
