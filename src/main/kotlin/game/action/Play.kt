package eelst.ilike.game.action

import eelst.ilike.game.entity.Slot


data class Play(
    val slot: Slot,
) : GameAction()
