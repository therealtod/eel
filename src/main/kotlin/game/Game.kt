package eelst.ilike.game

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

interface Game {
    /**
     * Set the given [gameState] as the starting state of the [Game]
     */
    fun setInitialGameState(gameState: GameState): Game

    /**
     * @return this [Game] after a player has played [playedCard]
     */
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean = false): Game

    /**
     * @return this [Game] after a player has discarded [discardedCard]
     */
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): Game

    /**
     * @return this [Game] after a player has given a clue
     */
    fun getAfter(clueAction: ClueAction, touchedSlotIndexes: Set<Int>): Game
}