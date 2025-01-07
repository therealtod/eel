package eelst.ilike.game.entity.suit


import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank

abstract class Suit(
    val id: SuitId,
    val name: String,
    val abbreviations: List<String>,
    specialRanks: Set<Rank> = emptySet(),
    stackSize: Int,
    val suitDirection: SuitDirection = SuitDirection.UP,
) {
    private val ranks: Set<Rank> = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)
        .filter { it.numericalValue <= stackSize }
        .plus(specialRanks)
        .toSet()

    val maxRank = ranks.last()

    // For hashing purposes
    private val copiesValuesList = Rank.entries.map { copiesOf(it) }

    fun getAllUniqueCards(): List<HanabiCard> {
        return ranks.map {
            HanabiCard(
                suit = this,
                rank = it
            )
        }
    }

    fun getAllCards(): List<HanabiCard> {
        return getAllUniqueCards().flatMap { card ->
            copiesOf(card.rank).downTo(0).map { card }
        }
    }

    fun copiesOf(rank: Rank): Int {
        return when (rank) {
            Rank.ONE -> 3
            Rank.FIVE -> 1
            else -> 2
        }
    }

    abstract fun getAssociatedColors(): Collection<Color>

    open fun clueTouches(card: HanabiCard, clueValue: ClueValue): Boolean {
        return when (clueValue) {
            is Rank -> return cluedRankTouches(card.rank, clueValue)
            is Color -> return cluedColorTouches(card.rank, clueValue)
            else -> throw IllegalArgumentException("Unsupported clue value $clueValue")
        }
    }

    /*
    fun cardAfter(card: HanabiCard): HanabiCard {
        val nextRank = ranks.firstOrNull { it > card.rank }
            ?: throw IllegalArgumentException("$card is the last card for suit $this")
        return HanabiCard(
            suit = this,
            rank = nextRank
        )
    }

    fun cardBefore(card: HanabiCard): HanabiCard {
        val nextRank = ranks.lastOrNull { it < card.rank }
            ?: throw IllegalArgumentException("$card is the first card for suit $this")
        return HanabiCard(
            suit = this,
            rank = nextRank
        )
    }
     */

    /*
    open fun getCardsBefore(card: HanabiCard): List<HanabiCard> {
        return ranks.filter { it < card.rank }
            .map { HanabiCard(suit = this, rank = it) }
    }

    open fun getCardsBetween(firstCard: HanabiCard, secondCard: HanabiCard): Set<HanabiCard> {
        return ranks.filter { it > firstCard.rank && it < secondCard.rank }
            .map { HanabiCard(suit = this, rank = it) }
            .toSet()
    }

     */

    abstract fun cluedRankTouches(thisSuitRank: Rank, cluedRank: Rank): Boolean

    abstract fun cluedColorTouches(thisSuitRank: Rank, cluedColor: Color): Boolean

    abstract fun getPlayingOrder(card: HanabiCard): Int

    override fun equals(other: Any?): Boolean {
        if (other == null) return false
        return if (other is Suit) {
            name == other.name
                    && maxRank == other.maxRank
                    && suitDirection == other.suitDirection
                    && copiesValuesList == other.copiesValuesList
        } else false
    }

    override fun hashCode(): Int {
        var result = name.hashCode()
        result = 31 * result + abbreviations.hashCode()
        result = 31 * result + maxRank.hashCode()
        result = 31 * result + suitDirection.hashCode()
        result = 31 * result + copiesValuesList.hashCode()
        return result
    }

    override fun toString(): String {
        return name
    }
}
