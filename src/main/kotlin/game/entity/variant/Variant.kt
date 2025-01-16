package eelst.ilike.game.entity.variant

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.suit.Suit


interface Variant {
    val id: String
    val name: String

    /**
     * @return all the [Suit] that compose this [Variant]
     */
    fun getSuits(): List<Suit>

    /**
     * @return all possible clue values that can be given in a game with this variant
     */
    fun getClueValues(): List<ClueValue>

    /**
     * @return the maximum score obtainable in a game of this [Variant]
     */
    fun getMaxScore(): Int
}
