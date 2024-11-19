package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.card.HanabiCard

sealed class Clue (val value: Any) {
    abstract fun touches(card: HanabiCard): Boolean
}
