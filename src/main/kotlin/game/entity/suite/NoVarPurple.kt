package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color


object NoVarPurple: ClassicSuite(
    id = "no_var_purple",
    name = "purple",
    abbreviations = setOf('p'),
) {
    override val suiteColors = setOf(Color.PURPLE)
}