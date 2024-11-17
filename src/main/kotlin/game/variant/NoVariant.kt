package eelst.ilike.game.variant

import eelst.ilike.game.entity.suite.Blue
import eelst.ilike.game.entity.suite.Green
import eelst.ilike.game.entity.suite.Purple
import eelst.ilike.game.entity.suite.Red
import eelst.ilike.game.entity.suite.Yellow


object NoVariant
    : Variant(
        name = "No Variant",
        suites = setOf(
            Red,
            Yellow,
            Green,
            Blue,
            Purple,
        )
    )
