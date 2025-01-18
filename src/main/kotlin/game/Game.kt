package eelst.ilike.game

import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.game.gamestate.GameState

interface Game {
    /**
     * @return the [Variant] set for this [Game]
     */
    fun getVariant(): Variant

    /**
     * @return a [List] of the [PlayerMetadata] of the players participating in this [Game]
     */
    fun getPlayersMetadata(): List<PlayerMetadata>

    /**
     * @return the [GloballyAvailableGameData] associated to the latest game state
     */
    fun getGloballyAvailableGameData(): GloballyAvailableGameData

    /**
     * @return the latest (current) [GameState] of this game
     */
    fun getCurrentGameState(): GameState

    /**
     * @return this [Game] after a player draws a card
     */
    fun getAfter(drawAction: DrawAction): Game

    /**
     * @return this [Game] after a player plays a card
     */
    fun getAfter(playAction: PlayAction): Game

    /**
     * @return this [Game] after a player discards a card
     */
    fun getAfter(discardAction: DiscardAction): Game

    /**
     * @return this [Game] after a player has given a clue
     */
    fun getAfter(clueAction: ClueAction, touchedSlotIndexes: Set<Int>): Game
}
