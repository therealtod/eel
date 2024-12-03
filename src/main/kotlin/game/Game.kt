package eelst.ilike.game

import eelst.ilike.game.entity.GameEvent
import eelst.ilike.game.entity.Player

interface Game {
    fun getPlayer(playerId: PlayerId): Player
    fun getEvents(): List<GameEvent>
}
