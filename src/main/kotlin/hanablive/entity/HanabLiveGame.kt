package eelst.ilike.hanablive.entity

import eelst.ilike.game.Game
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameActionData

class HanabLiveGame: Game {
    override fun setInitialGameState(gameState: GameState): Game {
        require(gameStates.isEmpty()) {
            "Cannot set the initial state of a game that has been already initialized"
        }
        gameStates.add(gameState)
        return this
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(playAction, playedCard, isStrike)
        gameStates.add(newGameState)
        return this
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(discardAction, discardedCard)
        gameStates.add(newGameState)
        return this
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotIndexes: Set<Int>): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(clueAction, touchedSlotIndexes)
        gameStates.add(newGameState)
        return this
    }

    private val gameStates: MutableList<GameState> = mutableListOf()
    private val actions: MutableList<HanabLiveGameActionData> = mutableListOf()
}
