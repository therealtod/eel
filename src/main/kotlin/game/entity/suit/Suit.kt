package eelst.ilike.game.entity.suit


import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank

interface Suit {
    val id: SuitId
    val name: String
    val abbreviations: List<String>
    val ranks: Set<Rank>
    val suitDirection: SuitDirection

    /**
     * @return a [Set] containing each unique [HanabiCard] that are contained in this [Suit]
     */
    fun getAllUniqueSuitCards(): Set<HanabiCard>

    /**
     * @return all the [HanabiCard] instances that compose this [Suit]
     */
    fun getAllSuitCards(): List<HanabiCard>

    /**
     * @return how many copies of the given [rank] are contained among the cards of this [Suit]
     */
    fun copiesOf(rank: Rank): Int

    /**
     * @return true if the given a clue with a given [clueValue] would touch the card of this [Suit] of the given [rank]
     */
    fun rankIsTouchedBy(rank: Rank, clueValue: ClueValue): Boolean

    /**
     * @return the [Color] clues that are available in a game as a consequence of this [Suit] being part of it
     */
    fun getAssociatedColors(): List<Color>

    /**
     * @return a [Int] n meaning this card needs to be played as the nth card of the suit
     */
    fun getPlayingOrder(card: HanabiCard): Int

    /**
     * @return the [Rank] that by being played makes a playing stack of this suit complete
     */
    fun getLastRank(): Rank
}
