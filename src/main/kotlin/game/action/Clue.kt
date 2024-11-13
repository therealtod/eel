package eelst.ilike.game.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

sealed class Clue (
    val value: Any,
    open val receiver: PlayerId,
): GameAction() {
    abstract fun touches(card: HanabiCard): Boolean
}
