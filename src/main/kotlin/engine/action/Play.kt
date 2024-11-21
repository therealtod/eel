package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId

data class Play(
    override val from: PlayerId,
    val slotIndex: Int,
) : PlayerAction(from)
