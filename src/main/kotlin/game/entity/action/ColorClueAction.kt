package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard

class ColorClueAction(
    clueGiver: PlayerId,
    clueReceiver: PlayerId,
    val color: Color
): ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = color
) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getColorsTouching(card.rank).contains(color)
    }
}
