package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction

/**
 * A [ConventionTech] associated with the action of playing
 */
interface PlayTech : ConventionTech {
    override fun matches(discardAction: DiscardAction, gameState: GameState): Boolean {
        return false
    }

    override fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, gameState: GameState): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(discardAction: DiscardAction, gameState: GameState): TeamKnowledge {
        return KnowledgeFactory.createEmptyTeamKnowledge(gameState)
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        gameState: GameState
    ): TeamKnowledge {
        return KnowledgeFactory.createEmptyTeamKnowledge(gameState)
    }
}
