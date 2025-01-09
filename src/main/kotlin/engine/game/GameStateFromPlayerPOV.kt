package eelst.ilike.engine.game

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId

/**
 * A [GameState] as a specific [Player] would see it from their point of view
 */
abstract class GameStateFromPlayerPOV(
    override val globallyAvailableGameData: GloballyAvailableGameData,
    override val players: Map<PlayerId, Player>,
    private val teamKnowledge: TeamKnowledge,
): GameState
