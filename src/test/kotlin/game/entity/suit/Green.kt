package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.suit.ClassicSuit

object Green : ClassicSuit(
    id = "green",
    name = "green",
    abbreviations = listOf("g", "G", "green"),
    definingColor = Color.GREEN,
)
