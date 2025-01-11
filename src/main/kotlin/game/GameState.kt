package eelst.ilike.game

import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId


interface GameState {
    val globallyAvailableGameData: GloballyAvailableGameData
    val players: Map<PlayerId, Player>
    val defaultHandsSize: Int
    val numberOfPlayers: Int

    /**
     * @return the [Player] playing in this game with the given [playerId]
     */
    fun getPlayer(playerId: PlayerId): Player

    /**
     * @return the [Player] playing in this game with the given [playerIndex]
     */
    fun getPlayer(playerIndex: Int): Player

    /**
     * @return all the possible options available to a player when giving a clue
     */
    fun getAvailableClueValues(): Set<ClueValue>

    /**
     * @return the new [GameState] after a [Player] draws a new slot
     */
    fun getAfter(drawAction: DrawAction): GameState

    /**
     * @param isStrike is merely a validator telling if the played card caused a strike
     *
     * @return the new [GameState] after a [Player] plays the given [playedCard]
     */
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean = false): GameState

    /**
     * @return the new [GameState] after a [Player] discards the given [discardedCard]
     */
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState

    /**
     * return the new [GameState] after a [Player] gives a clue to a teammate, touching the slots with
     * indexes [touchedSlotsIndexes]
     */
    fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>): GameState
}
