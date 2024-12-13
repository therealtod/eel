package eelst.ilike.engine.hand.slot

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class KnownSlot(
    index: Int,
    positiveClues: List<ClueValue>,
    negativeClues: List<ClueValue>,
    val inferredIdentity: HanabiCard,
): BaseSlot(
    index = index,
    positiveClues = positiveClues,
    negativeClues = negativeClues,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return card == inferredIdentity
    }
}
