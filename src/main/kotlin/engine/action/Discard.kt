package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId

data class Discard(override val from: PlayerId, val slotIndex: Int) : GameAction(from)
