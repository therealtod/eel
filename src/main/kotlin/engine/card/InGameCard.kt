package eelst.ilike.engine.card

import eelst.ilike.engine.knowledge.PlayerKnowledge
import eelst.ilike.engine.knowledge.SlotKnowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.player.PlayerId


class InGameCard(
    val positionInStartingDeck: Int,
    val positiveClues: MutableList<ClueValue> = mutableListOf(),
    val negativeClues: MutableList<ClueValue> = mutableListOf(),
    val slotKnowledge: SlotKnowledge,
) {
    fun updateWith(other: InGameCard): InGameCard{
        TODO()
    }
}
