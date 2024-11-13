package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object NoVarRed: ClassicSuite(
    id = "no_var_red",
    name = "red",
    abbreviations = setOf('r'),
) {
    override val suiteColors = setOf(Color.RED)
}
