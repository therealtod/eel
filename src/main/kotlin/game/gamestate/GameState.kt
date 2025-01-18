package eelst.ilike.game.gamestate

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction


interface GameState {
    val globallyAvailableGameData: GloballyAvailableGameData

    /**
     * @return the new [GameState] after a player draws an unspecified card (and adds it to slot 1)
     */
    fun getAfter(drawAction: DrawAction): GameState

    /**
     * @return the new [GameState] after a player draws a specific [card] (and adds it to slot 1)
     */
    fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState

    /**
     * @return the new [GameState] after a player plays one of their slots (the played card is not specified)
     */
    fun getAfter(playAction: PlayAction): GameState

    /**
     * @return the new [GameState] after a player plays the given [playedCard]
     */
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState

    /**
     * @return the new [GameState] after a player discards one of their slots (the discarded card is not specified)
     */
    fun getAfter(discardAction: DiscardAction): GameState

    /**
     * @return the new [GameState] after a player discards the given [discardedCard]
     */
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState

    /**
     * return the new [GameState] after a player gives a clue to a teammate, touching the slots with
     * indexes [touchedSlotsIndexes]
     */
    fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState
}
