package eelst.ilike.game.entity

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

interface Player<T: Hand> {
    val playerId: PlayerId
    val playerIndex: Int
    val ownHand: T
}
