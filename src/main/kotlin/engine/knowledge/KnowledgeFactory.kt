package eelst.ilike.engine.knowledge

import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerId

object KnowledgeFactory {
    fun createEmptyTeamKnowledge(gameState: GameState): TeamKnowledge {
        TODO()
    }

    fun createPlayerKnowledge(
        playerId: PlayerId,
        globallyAvailableGameData: GloballyAvailableGameData,
        cardsVisibleInPlayerHands: Map<PlayerId, Map<Int, HanabiCard>>,
        inferredHandKnowledge: Map<PlayerId, InferredHandKnowledge>,
    ): PlayerKnowledge {
        val globallyVisibleCards = globallyAvailableGameData.getCardsOnStacks() +
                globallyAvailableGameData.trashPile.cards
        return PlayerKnowledgeImpl(
            playerId = playerId,
            globallyVisibleCards = globallyVisibleCards,
            cardsVisibleInPlayerHands = cardsVisibleInPlayerHands,
            inferredHandKnowledge = inferredHandKnowledge
        )
    }

    fun createTeamKnowledge(
        playersKnowledge: Map<PlayerId, PlayerKnowledge>
    ): TeamKnowledge {
        return TeamKnowledgeImpl(playersKnowledge)
    }

    fun createEmptyInferredHandKnowledge(): InferredHandKnowledge {
        return InferredHandKnowledgeImpl()
    }
}
