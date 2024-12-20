package eelst.ilike.game

import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard

interface Game {
    fun getGameData(): GameData
    fun getPlayers(): Map<PlayerId, Player>
    fun getPlayer(playerId: PlayerId): Player
    fun getPlayerMetadata(playerId: PlayerId): PlayerMetadata
    fun getPlayerMetadata(playerIndex: Int): PlayerMetadata
    fun getAfter(playAction: PlayAction, card: HanabiCard, successful: Boolean): Game
    fun getAfter(discardAction: DiscardAction): Game
    fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>)
}
