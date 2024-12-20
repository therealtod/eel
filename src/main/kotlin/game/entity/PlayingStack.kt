package eelst.ilike.game.entity

import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

data class PlayingStack(
    val cards: List<HanabiCard> = emptyList(),
    val suite: Suite,
) : List<HanabiCard> by cards {
    fun isComplete(): Boolean {
        return (cards.lastOrNull()?.rank) == suite.maxRank
    }

    fun currentCard(): HanabiCard {
        require(!isEmpty()) {
            "cannot retrieve the current cards for an empty stack"
        }
        return cards.last()
    }

    fun nextCard(card: HanabiCard): HanabiCard {
        require(card.suite == suite) {
            "The card $card does not belong to this stack"
        }
        return suite.cardAfter(card)
    }

    fun playCard(card: HanabiCard): PlayingStack {
        require(card.suite.id == suite.id) {
            "Wrong stack for card $card"
        }
        return PlayingStack(
            cards = cards + card,
            suite = suite
        )
    }
}
