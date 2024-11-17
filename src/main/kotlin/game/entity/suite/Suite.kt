package eelst.ilike.game.entity.suite

import eelst.ilike.game.action.Clue
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

abstract class Suite(
    val id: SuiteId,
    val name: String,
    val abbreviations: Set<Char>,
    val ranks: Set<Rank> = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE),
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

    open fun clueTouches(card: HanabiCard, clue: Clue): Boolean {
        return when(clue) {
            is RankClue -> getRanksTouching(card.rank).contains(clue.rank)
            is ColorClue -> getColorsTouching(card.rank).contains(clue.color)
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
        private val registeredSuites = mapOf(
            "no_var_red" to Red,
            "no_var_yellow" to Yellow,
            "no_var_green" to Green,
            "no_var_blue" to Blue,
            "no_var_purple" to Purple
        )

        fun fromId(suiteId: SuiteId): Suite {
            val registeredSuites = mapOf(
                "red" to Red,
                "yellow" to Yellow,
                "green" to Green,
                "blue" to Blue,
                "purple" to Purple
            )
            return registeredSuites[suiteId]
                ?: throw IllegalArgumentException("The suite with id $suiteId is unregistered")
        }

        fun fromAbbreviation(abbreviation: Char, suites: Set<Suite>): Suite {
            return suites.firstOrNull { it.abbreviations.contains(abbreviation) }
                ?: Unknown
        }
    }
}
