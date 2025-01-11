package eelst.ilike.game.entity.suit

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank
import org.apache.logging.log4j.kotlin.Logging

abstract class BaseSuit(
    override val abbreviations: List<String>,
) : Suit, Logging {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as BaseSuit

        return copiesOfMap == other.copiesOfMap && suitDirection == other.suitDirection
    }

    override fun hashCode(): Int {
        val result = copiesOfMap.hashCode()
        return result + 31 * suitDirection.hashCode()
    }

    override fun getAllSuitCards(): List<HanabiCard> {
        return allCards
    }

    override fun getAllUniqueSuitCards(): Set<HanabiCard> {
        return allUniqueCards
    }

    override fun copiesOf(rank: Rank): Int {
        val copiesAmount = copiesOfMap[rank]
        if (copiesAmount == null) {
            logger.warn("Unexpected: Suit.copiesOf() called with a non-existing rank $rank")
            return 0
        } else return copiesAmount
    }

    override fun toString(): String {
        return name
    }

    /**
     * [Map] associating how many copies of a certain [Rank] this [Suit] has
     */
    protected abstract val copiesOfMap: Map<Rank, Int>

    /**
     * All the cards that compose this suit
     */
    private val allCards: List<HanabiCard> by lazy {
        copiesOfMap.entries.flatMap {
            it.value.downTo(1).map { _ ->
                HanabiCard(
                    suit = this,
                    rank = it.key
                )
            }
        }
    }

    private val allUniqueCards by lazy {
        copiesOfMap.keys.map {
            HanabiCard(
                suit = this,
                rank = it,
            )
        }.toSet()
    }
}
