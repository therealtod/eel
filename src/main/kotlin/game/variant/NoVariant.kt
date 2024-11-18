package eelst.ilike.game.variant

import eelst.ilike.game.entity.suite.*


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
