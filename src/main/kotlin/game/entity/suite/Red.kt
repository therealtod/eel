package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Red: ClassicSuite(
    id = "red",
    name = "red",
    abbreviations = setOf('r'),
) {
    override val suiteColors = setOf(Color.RED)
}
