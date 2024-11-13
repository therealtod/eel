package eelst.ilike.game.variant

import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow


object NoVariant
    : Variant(
        name = "No Variant",
        suites = setOf(
            NoVarRed,
            NoVarYellow,
            NoVarGreen,
            NoVarBlue,
            NoVarPurple,
        )
    )
