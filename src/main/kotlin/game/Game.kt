package eelst.ilike.game

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.game.entity.Player

interface Game {
    fun getGameData(): GameData
    fun getPlayers(): Map<PlayerId, Player>
    fun getPlayer(playerId: PlayerId): Player
    fun getPlayerMetadata(playerId: PlayerId): PlayerMetadata
    fun getPlayerMetadata(playerIndex: Int): PlayerMetadata
    fun getAfter(observedAction: ObservedAction): Game
}
