package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard

data class ColorClueAction(
    override val clueGiver: PlayerId,
    override val clueReceiver: PlayerId,
    val color: Color
): ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = color
)
