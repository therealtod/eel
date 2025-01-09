package eelst.ilike.game.entity

import eelst.ilike.game.entity.suit.Suit

data class PlayingStack(
    val cards: List<HanabiCard> = emptyList(),
    val suit: Suit,
) : List<HanabiCard> by cards {
    /**
     * @return true if the stack is complete.
     */
    fun isComplete(): Boolean {
        return (cards.lastOrNull()?.rank) == suit.maxRank
    }

    /**
     * @return the [HanabiCard] currently on top of the stack
     * @throws [IllegalAccessException] when the stack is empty
     */
    fun currentCard(): HanabiCard {
        if (isEmpty()) {
            throw IllegalAccessException("cannot retrieve the current cards for an empty stack")
        }
        return cards.last()
    }

    /**
     * @return a [Collection] of [HanabiCard] that can be used as next card to be played on this [PlayingStack]
     * @throws [IllegalAccessException] when the stack is complete
     */
    fun nextCards(variant: Variant): Collection<HanabiCard> {
        if(isComplete()) {
            throw IllegalAccessException("There is no next card for a complete stack")
        }
        if (cards.isEmpty()) {
            return getPossibleFirstCards()
        }
        return listOf(
            HanabiCard(
                suit = suit,
                rank = Rank.getByNumericalValue(currentCard().rank.numericalValue + 1)
            )
        )
    }

    /**
     * @return a new [PlayingStack] which includes the given [card] on top if it's a valid play,
     * or the original stack otherwise
     */
    fun getAfterPlaying(card: HanabiCard, variant: Variant): PlayingStack {
        require(card.suit.id == suit.id) {
            "Wrong stack for card $card"
        }
        return if (nextCards(variant).contains(card)) {
            PlayingStack(
                cards = cards + card,
                suit = suit,
            )
        } else this
    }

    /**
     * @return a [Collection] of [HanabiCard] that can be used as
     */
    fun getPossibleFirstCards(): Collection<HanabiCard> {
        return listOf(
                HanabiCard(
                suit = suit,
                rank = Rank.ONE,
            ),
        )
    }

    val maxSize = 5
}
