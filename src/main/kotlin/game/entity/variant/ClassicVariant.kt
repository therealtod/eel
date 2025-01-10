package eelst.ilike.game.entity.variant

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suit.Suit

class ClassicVariant(
    override val id: String,
    override val name: String,
    private val suits: List<Suit>,
): Variant {
    override fun getSuits(): List<Suit> {
        return suits
    }

    override fun getClueValues(): List<ClueValue> {
        val rankClues = listOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)
        val colorClues = suits.flatMap { it.getAssociatedColors() }
        return colorClues + rankClues
    }
}
