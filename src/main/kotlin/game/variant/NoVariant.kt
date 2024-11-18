package eelst.ilike.game.variant

import eelst.ilike.game.entity.suite.*


data object NoVariant
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
