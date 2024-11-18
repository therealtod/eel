package eelst.ilike.game.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard

data class ColorClue(
    val color: Color,
    override val receiver: PlayerId
) : Clue(
    value = color,
    receiver = receiver,
) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getColorsTouching(card.rank).contains(color)
    }
}
