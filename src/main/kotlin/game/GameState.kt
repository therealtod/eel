package eelst.ilike.game

import eelst.ilike.engine.card.InGameCard
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot


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
     * @return all the possible options available to a player when giving a clue
     */
    fun getAvailableClueValues(): Set<ClueValue>

    /**
     * @return the new [GameState] after a [Player] draws an unspecified card (and adds it to slot 1)
     */
    fun getAfter(drawAction: DrawAction): GameState

    /**
     * @return the new [GameState] after a [Player] draws a specific [card] (and adds it to slot 1)
     */
    fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState

    /**
     * @return the new [GameState] after a [Player] plays one of their slots (the played card is not specified)
     */
    fun getAfter(playAction: PlayAction): GameState

    /**
     * @return the new [GameState] after a [Player] plays the given [playedCard]
     */
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState

    /**
     * @return the new [GameState] after a [Player] discards one of their slots (the discarded card is not specified)
     */
    fun getAfter(discardAction: DiscardAction): GameState

    /**
     * @return the new [GameState] after a [Player] discards the given [discardedCard]
     */
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState

    /**
     * return the new [GameState] after a [Player] gives a clue to a teammate, touching the slots with
     * indexes [touchedSlotsIndexes]
     */
    fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState
}
