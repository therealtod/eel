package eelst.ilike.hanablive.factory

import eelst.ilike.engine.game.KnowledgeAwareGameState
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId

object GameStateFactory {
    fun createGameState(
        globallyAvailableGameData: GloballyAvailableGameData,
        players: Map<PlayerId, Player>,
        teamKnowledge: TeamKnowledge,
    ): GameState {
        return KnowledgeAwareGameState(
            globallyAvailableGameData = globallyAvailableGameData,
            players = players,
            teamKnowledge = teamKnowledge,
        )
    }
}
