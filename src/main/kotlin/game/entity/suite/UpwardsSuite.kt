package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

abstract class UpwardsSuite(
    id: SuiteId,
    name: String,
    abbreviations: Set<Char>,

): Suite(
    id = id,
    name = name,
    abbreviations = abbreviations,
    suiteDirection = SuiteDirection.UP
) {
    override fun cardAfter(card: HanabiCard): HanabiCard {
        require(card.suite == this) {
            "Can only determine the next card from a card belonging to this suite"
        }
        require(card.rank < this.maxRank) {
            "There is not next card after the max rank card of this suite"
        }
        return HanabiCard(
            suite = this,
            rank = Rank.getByNumericalValue(card.rank.numericalValue + 1)
        )
    }

    override fun cardBefore(card: HanabiCard): HanabiCard {
        require(card.suite == this) {
            "Can only determine the next card from a card belonging to this suite"
        }
        require(card.rank > Rank.ONE) {
            "There is not card that comes before a one for this suite"
        }
        return HanabiCard(
            suite = this,
            rank = Rank.getByNumericalValue(card.rank.numericalValue -1)
        )
    }

    override fun getCardsBefore(card: HanabiCard): List<HanabiCard> {
        require(card.suite == this) {
            "The given card must belong to this suite"
        }
        return getAllUniqueCards().filter { it.rank < card.rank}
    }

    override fun getCardsBetween(firstCard: HanabiCard, secondCard: HanabiCard): Set<HanabiCard> {
        require(firstCard.suite == this && secondCard.suite == this) {
            "This operation is only supported for cards belonging to this suite"
        }
        require(firstCard.rank < secondCard.rank) {
            "The first card needs to be played before the second card for this suite"
        }
        val cardsBetween = mutableListOf<HanabiCard>()
        var currentCard = cardAfter(firstCard)
        while (currentCard != secondCard) {
            cardsBetween.add(currentCard)
            currentCard = cardAfter(currentCard)
        }
        return cardsBetween.toSet()
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        require(card.suite == this) {
            "Cannot determine the playing order of $card because it does not belong to this suite"
        }
        return card.rank.numericalValue
    }
}