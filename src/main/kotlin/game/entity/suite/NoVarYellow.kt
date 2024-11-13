package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object NoVarYellow: ClassicSuite(
    id = "no_var_yellow",
    name = "yellow",
    abbreviations = setOf('y'),
) {
    override val suiteColors = setOf(Color.YELLOW)
}
