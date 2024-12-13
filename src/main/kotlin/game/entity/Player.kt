package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId

interface Player {
    val playerId: PlayerId
    val playerIndex: Int
    val hand: Hand

    fun getSlots(): Set<Slot>
}
