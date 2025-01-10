package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.suit.ClassicSuit

object Yellow : ClassicSuit(
    id = "yellow",
    name = "yellow",
    abbreviations = listOf("y", "Y", "Yellow"),
    definingColor = Color.YELLOW,
)
