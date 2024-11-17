package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object Blue: ClassicSuite(
    id = "blue",
    name = "blue",
    abbreviations = setOf('b'),
) {
    override val suiteColors = setOf(Color.BLUE)
}
