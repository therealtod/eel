package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object Purple : ClassicSuite(
    id = "purple",
    name = "purple",
    abbreviations = listOf("p"),
) {
    override val suiteColors = setOf(Color.PURPLE)
}