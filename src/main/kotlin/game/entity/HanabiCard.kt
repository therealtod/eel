package eelst.ilike.game.entity

import eelst.ilike.game.entity.suit.Suit


data class HanabiCard(
    val suit: Suit,
    val rank: Rank,
) {
    fun isTouchedBy(clueValue: ClueValue): Boolean {
        return suit.rankIsTouchedBy(rank, clueValue)
    }
}
