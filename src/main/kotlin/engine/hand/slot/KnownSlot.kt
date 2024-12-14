package eelst.ilike.engine.hand.slot

import eelst.ilike.game.entity.BaseSlot
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard

open class KnownSlot(
    index: Int,
    positiveClues: List<ClueValue>,
    negativeClues: List<ClueValue>,
    val knownIdentity: HanabiCard,
): BaseSlot(
    index = index,
    positiveClues = positiveClues,
    negativeClues = negativeClues,
) {
    override fun containsCard(card: HanabiCard): Boolean {
        return card == knownIdentity
    }
}
