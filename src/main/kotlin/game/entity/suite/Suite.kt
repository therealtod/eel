package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.utils.Configuration

abstract class Suite(
    val id: SuiteId,
    val name: String,
    val abbreviations: Set<Char>,
    private val ranks: Set<Rank> = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE),
    val suiteDirection: SuiteDirection = SuiteDirection.UP,
) {
    val maxRank = ranks.last()

    // For hashing purposes
    private val copiesValuesList = Rank.entries.map { copiesOf(it) }

    fun getAllUniqueCards(): List<HanabiCard> {
        return ranks.map {
            HanabiCard(
                suite = this,
                rank = it
            )
        }
    }

    fun copiesOf(rank: Rank): Int {
        return when (rank) {
            Rank.ONE -> 3
            Rank.FIVE -> 1
            else -> 2
        }
    }

    open fun clueTouches(card: HanabiCard, clue: ClueValue): Boolean {
        return when (clue) {
            is Rank -> getRanksTouching(card.rank).contains(clue.rank)
            is Color -> getColorsTouching(card.rank).contains(clue.color)
        }
    }

    abstract fun cardAfter(card: HanabiCard): HanabiCard

    abstract fun cardBefore(card: HanabiCard): HanabiCard

    abstract fun getCardsBefore(card: HanabiCard): List<HanabiCard>

    abstract fun getCardsBetween(firstCard: HanabiCard, secondCard: HanabiCard): Set<HanabiCard>

    abstract fun getRanksTouching(rank: Rank): Set<Rank>

    abstract fun getColorsTouching(rank: Rank): Set<Color>

    abstract fun getPlayingOrder(card: HanabiCard): Int

    override fun equals(other: Any?): Boolean {
        if (other == null) return false
        return if (other is Suite) {
            name == other.name
                    && maxRank == other.maxRank
                    && suiteDirection == other.suiteDirection
                    && copiesValuesList == other.copiesValuesList
        } else false
    }

    override fun hashCode(): Int {
        var result = name.hashCode()
        result = 31 * result + abbreviations.hashCode()
        result = 31 * result + maxRank.hashCode()
        result = 31 * result + suiteDirection.hashCode()
        result = 31 * result + copiesValuesList.hashCode()
        return result
    }

    companion object {
        fun fromId(suiteId: SuiteId): Suite {
            return Configuration.registeredSuitesMap[suiteId]
                ?: throw IllegalArgumentException("The suite with id $suiteId is unregistered")
        }

        fun fromAbbreviation(abbreviation: Char, suites: Set<Suite>): Suite {
            return suites.firstOrNull { it.abbreviations.contains(abbreviation) }
                ?: Unknown
        }
    }

    override fun toString(): String {
        return name
    }
}
