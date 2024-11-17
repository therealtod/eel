package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId

interface Player<T : Hand<*>> {
    val playerId: PlayerId
    val playerIndex: Int
    val ownHand: T
}
