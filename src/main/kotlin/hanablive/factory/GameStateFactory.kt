package eelst.ilike.hanablive.factory

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.GameStateImpl
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId

object GameStateFactory {
    fun createGameState(
        globallyAvailableGameData: GloballyAvailableGameData,
        players: Map<PlayerId, Player>,
    ): GameState {
        return GameStateImpl(
            globallyAvailableGameData = globallyAvailableGameData,
            players = players,
        )
    }
}
