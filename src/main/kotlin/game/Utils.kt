package eelst.ilike.game

import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

object Utils {
    fun getCardEmpathy(
        visibleCards: List<HanabiCard>,
        positiveClues: List<Clue>,
        negativeClues: List<Clue>,
        suites: Set<Suite>
    ): Set<HanabiCard> {
        val allPossibleCards = suites.flatMap { suite ->
            suite.getAllUniqueCards()
        }
        return allPossibleCards
            .filter { card ->
                // Exclude all the cards whose copies are all visible
                visibleCards.count { it == card } < card.suite.copiesOf(card.rank)
            }
            .filter { card ->

                positiveClues.all { clue ->
                    // Exclude all the cards which cannot be touched by the clues the slot was touched by
                    card.suite.clueTouches(card, clue)
                } && negativeClues.none { clue ->
                    // Exclude all the cards which can be touched by the clues the slot was not touched by
                    card.suite.clueTouches(card, clue)
                }
            }
            .toSet()
    }
}
