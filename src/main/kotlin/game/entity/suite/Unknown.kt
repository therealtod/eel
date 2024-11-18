package eelst.ilike.game.entity.suite

import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

object Unknown : Suite(
    id = "unknown",
    name = "Unknown",
    abbreviations = setOf('x'),
) {
    override fun clueTouches(
        card: HanabiCard,
        clue: Clue
    ): Boolean {
        return super.clueTouches(card, clue)
    }

    override fun cardAfter(card: HanabiCard): HanabiCard {
        TODO("Not yet implemented")
    }

    override fun cardBefore(card: HanabiCard): HanabiCard {
        TODO("Not yet implemented")
    }

    override fun getCardsBefore(card: HanabiCard): List<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getCardsBetween(
        firstCard: HanabiCard,
        secondCard: HanabiCard
    ): Set<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getRanksTouching(rank: Rank): Set<Rank> {
        TODO("Not yet implemented")
    }

    override fun getColorsTouching(rank: Rank): Set<Color> {
        TODO("Not yet implemented")
    }

    override fun getPlayingOrder(card: HanabiCard): Int {
        TODO("Not yet implemented")
    }
}