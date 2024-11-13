package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object NoVarGreen: ClassicSuite(
    id = "no_var_green",
    name = "green",
    abbreviations = setOf('g'),
) {
    override val suiteColors = setOf(Color.GREEN)
}
