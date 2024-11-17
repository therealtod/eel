package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Yellow: ClassicSuite(
    id = "yellow",
    name = "yellow",
    abbreviations = setOf('y'),
) {
    override val suiteColors = setOf(Color.YELLOW)
}
