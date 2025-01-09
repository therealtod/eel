package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

/**
 * A [ConventionTech] associated with the action of cluing
 */
interface ClueTech : ConventionTech {
    override fun matches(playAction: PlayAction, gameState: GameState): Boolean {
        return false
    }

    override fun matches(discardAction: DiscardAction, gameState: GameState): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(playAction: PlayAction, gameState: GameState): TeamKnowledge {
        return KnowledgeFactory.createEmptyTeamKnowledge(gameState)
    }

    override fun getGeneratedKnowledge(discardAction: DiscardAction, gameState: GameState): TeamKnowledge {
        return KnowledgeFactory.createEmptyTeamKnowledge(gameState)
    }
}
