package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.card.HanabiCard

class ColorClueAction(
    clueGiver: PlayerId,
    clueReceiver: PlayerId,
    color: Color
): ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = color
)
