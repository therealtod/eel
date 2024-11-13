package eelst.ilike.engine

import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

object EngineHelper {
    fun getCardEmpathy(
        playerPOV: PlayerPOV,
        positiveClues: List<Clue>,
        negativeClues: List<Clue>,
    ): Set<HanabiCard> {
        val suites = playerPOV.globallyAvailableInfo.playingStacks.map { it.value.suite }
        val allPossibleCards = suites.flatMap { suite ->
            suite.getAllUniqueCards()
        }
        val visibleCards = playerPOV.getVisibleCards()
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