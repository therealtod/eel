package eelst.ilike.engine.card

import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.suit.Suit
import engine.card.CardLocationDictionary

/**
 * Represents the aggregated information as seen by the players during the game on all the cards
 */
class InGameCardsAggregatedData(
    suits: Collection<Suit>,
    playersMetadata: Collection<PlayerMetadata>,
) {
    val cards = List(
        suits
            .flatMap { it.getAllSuitCards() }.size) { index->
        InGameCard(
            positionInStartingDeck = index,
            slotKnowledge = KnowledgeFactory.createEmptySlotKnowledge(playersMetadata)
        )
    }.toMutableList()

    fun updateWith(drawAction: DrawAction): InGameCardsAggregatedData {
        TODO()
    }

    fun updateWith(playAction: PlayAction): InGameCardsAggregatedData {
        TODO()
    }

    fun updateWith(discardAction: DiscardAction): InGameCardsAggregatedData {
        TODO()
    }

    fun updateWith(clueAction: ClueAction): InGameCardsAggregatedData {
        TODO()
    }

    fun getCardLocationDictionary(): CardLocationDictionary {
        TODO()
    }
}
