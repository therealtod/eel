package game.entity.variant

import eelst.ilike.game.entity.variant.ClassicVariant
import game.entity.suit.*

object NoVariant : ClassicVariant(
    id = "No Variant",
    name = "No Variant",
    suits = listOf(Red, Yellow, Green, Blue, Purple),
)
