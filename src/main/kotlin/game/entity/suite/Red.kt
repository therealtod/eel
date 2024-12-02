package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Red : ClassicSuite(
    id = "red",
    name = "red",
    abbreviations = listOf("r"),
) {
    override val suiteColors = setOf(Color.RED)
}
