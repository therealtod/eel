package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

object NoVarBlue: ClassicSuite(
    id = "no_var_blue",
    name = "blue",
    abbreviations = setOf('b'),
) {
    override val suiteColors = setOf(Color.BLUE)
}
