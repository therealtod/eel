package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId

interface Player<T: Slot> {
    val playerId: PlayerId
    val playerIndex: Int

    fun getHand(): Hand<T>
}
