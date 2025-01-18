package eelst.ilike.game.gamestate

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction

class BaseGameState(
    override val globallyAvailableGameData: GloballyAvailableGameData,
): GameState {
    override fun getAfter(drawAction: DrawAction): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDraw(drawAction)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDraw(drawAction)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(playAction: PlayAction): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterPlay(playAction)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterPlaying(playAction, playedCard)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(discardAction: DiscardAction): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDiscard(discardAction)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDiscarding(discardAction, discardedCard)
        return BaseGameState(newGloballyAvailableGameData)
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterClueGiven(clueAction, touchedSlotsIndexes)
        return BaseGameState(newGloballyAvailableGameData)
    }
}