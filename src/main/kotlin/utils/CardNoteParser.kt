package eelst.ilike.utils

import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

object CardNoteParser {
    fun parseCards(text: String, suites: Set<Suite>): Set<HanabiCard> {
        val cardAbbreviations = text.chunked(2)
        return cardAbbreviations.map {
            parseCard(it, suites)
        }.toSet()
    }

    fun parseCard(cardAbbreviation: String, suites: Set<Suite>): HanabiCard {
        val suiteAbbreviation = cardAbbreviation.first()
        val rank = Rank.getByNumericalValue(cardAbbreviation.last().toString().toInt())
        val suite = Suite.fromAbbreviation(suiteAbbreviation, suites)
        return HanabiCard(
            suite = suite,
            rank = rank,
        )
    }

    fun parseClue(clueAbbreviation: String, suites: Set<Suite>): Clue{
        TODO()
    }
}
