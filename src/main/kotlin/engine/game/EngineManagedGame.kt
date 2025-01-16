package eelst.ilike.engine.game

import eelst.ilike.engine.card.InGameCardsAggregatedData
import eelst.ilike.game.Game
import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.variant.Variant

/*
open class EngineManagedGame(
    variant: Variant,
    playersMetadata: List<PlayerMetadata>
): Game {
    /*
    protected val globallyAvailableGameData = GloballyAvailableGameData(
        variant = variant,
        playingStacks = variant.getSuits().associate { it.id to PlayingStack(emptyList(), it) },
        trashPile = TrashPile(),
        strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
        clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
        amountOfCardsPlayed = 0,
        playersMetadata = playersMetadata,
    )

     */

    override fun getGloballyAvailableGameData(): GloballyAvailableGameData {
        return gameStates.last().globallyAvailableGameData
    }

    override fun getCurrentGameState(): GameState {
        return gameStates.last()
    }

    override fun getAfter(drawAction: DrawAction): Game {
        cardsAggregatedData.updateWith(drawAction)
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(drawAction)
        gameStates.addFirst(newGameState)
        return this
    }

    override fun getAfter(playAction: PlayAction): Game {
        cardsAggregatedData.updateWith(playAction)
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(playAction)
        gameStates.addFirst(newGameState)
        return this
    }

    override fun getAfter(discardAction: DiscardAction): Game {
        cardsAggregatedData.updateWith(discardAction)
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(discardAction)
        gameStates.addFirst(newGameState)
        return this
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotIndexes: Set<Int>): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(clueAction, touchedSlotIndexes)
        gameStates.add(newGameState)
        return this
    }



}

 */