package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Green: ClassicSuite(
    id = "green",
    name = "green",
    abbreviations = setOf('g'),
) {
    override val suiteColors = setOf(Color.GREEN)
}
