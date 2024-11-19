package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard

data class ColorClue(val color: Color): Clue(color) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getColorsTouching(card.rank).contains(color)
    }
}
