package eelst.ilike.game.action

import eelst.ilike.game.Slot


data class Play(
    val slot: Slot,
) : GameAction()
