package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.clue.Clue

data class ColorClue(val color: Color) : Clue(color) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getColorsTouching(card.rank).contains(color)
    }
}
