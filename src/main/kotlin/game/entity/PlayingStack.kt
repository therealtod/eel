package eelst.ilike.game.entity

import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suit

data class PlayingStack(
    val cards: List<HanabiCard> = emptyList(),
    val suit: Suit,
) : List<HanabiCard> by cards {
    fun isComplete(): Boolean {
        return (cards.lastOrNull()?.rank) == suit.maxRank
    }

    fun currentCard(): HanabiCard {
        require(!isEmpty()) {
            "cannot retrieve the current cards for an empty stack"
        }
        return cards.last()
    }

    fun nextCard(card: HanabiCard): HanabiCard {
        require(card.suit == suit) {
            "The card $card does not belong to this stack"
        }
        return suit.cardAfter(card)
    }

    fun playCard(card: HanabiCard): PlayingStack {
        require(card.suit.id == suit.id) {
            "Wrong stack for card $card"
        }
        return PlayingStack(
            cards = cards + card,
            suit = suit
        )
    }
}
