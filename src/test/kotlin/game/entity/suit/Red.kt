package game.entity.suit

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.suit.ClassicSuit

object Red : ClassicSuit(
    id = "red",
    name = "red",
    abbreviations = listOf("r", "R", "red"),
    definingColor = Color.RED
)
