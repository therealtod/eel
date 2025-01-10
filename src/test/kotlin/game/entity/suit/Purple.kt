package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.suit.ClassicSuit

object Purple : ClassicSuit(
    id = "purple",
    name = "purple",
    abbreviations = listOf("p", "P", "purple"),
    definingColor = Color.PURPLE,
)
