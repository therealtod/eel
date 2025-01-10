package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.suit.ClassicSuit

object Blue : ClassicSuit(
    id = "blue",
    name = "blue",
    abbreviations = listOf("b", "B", "blue"),
    definingColor = Color.BLUE,
)
