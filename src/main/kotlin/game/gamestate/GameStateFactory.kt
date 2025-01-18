package eelst.ilike.game.gamestate

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.game.KnowledgeAwareGameState
import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.GloballyAvailableGameDataFactory
import eelst.ilike.game.entity.player.PlayerMetadata

object GameStateFactory {
    fun createKnowledgeAwareGameState(
        globallyAvailableGameData: GloballyAvailableGameData,
        conventionSet: ConventionSet,
    ): GameState {
        val base = BaseGameState(globallyAvailableGameData)
        val teamKnowledge = KnowledgeFactory.createEmptyTeamKnowledge(globallyAvailableGameData)
        return KnowledgeAwareGameState(
            gameState = base,
            teamKnowledge = teamKnowledge,
            conventionSet = conventionSet,
        )
    }
}
