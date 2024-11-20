package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

sealed class Clue(val value: ClueValue) {
    abstract fun touches(card: HanabiCard): Boolean
}
