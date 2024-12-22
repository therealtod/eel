package eelst.ilike.hanablive.bot.state

import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.adapter.HanabLivePlayerPOVAdapter
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.*

class PlayingState(
    bot: HanabLiveBot,
    commonState: CommonState,
    private var game: HanabLivePlayerPOVAdapter,
): HanabLiveBotState(
    bot =  bot,
    commonState = commonState,
) {
    private val hanabLiveActions = mutableListOf<HanabLiveGameAction>()

    override suspend fun onGameAction(gameAction: HanabLiveGameAction) {
        hanabLiveActions.add(gameAction)
        val turnActionReceived = hanabLiveActions.any {
            it.action.type == GameActionType.TURN
        }
        if (turnActionReceived) {
            handleAction(hanabLiveActions)
            hanabLiveActions.clear()
        }
    }

    private fun handleAction(hanabLiveGameActions: List<HanabLiveGameAction>) {
        require(hanabLiveGameActions.count { HanabLiveConstants.PLAYER_ACTIONS.contains(it.action.type) } != 1) {
            "Only one player action should be included in this bundle: $hanabLiveGameActions"
        }
        val playerAction = hanabLiveGameActions.first { HanabLiveConstants.PLAYER_ACTIONS.contains(it.action.type) }
        val playerActionData = playerAction.action
        val isStrike = hanabLiveGameActions.any { it.action.type == GameActionType.STRIKE }
        game = when (playerActionData.type) {
            GameActionType.DRAW -> getPOVAfterDraw(playerActionData as GameDrawActionData)
            GameActionType.PLAY -> getPOVAfterPlay(playerActionData as GamePlayActionData, isStrike)
            GameActionType.DISCARD -> getPOVAfterDiscard(playerActionData as GameDiscardActionData)
            GameActionType.CLUE -> getPOVAfterClue(playerActionData as GameClueActionData)
            else -> game
        }
        val turnActionData = hanabLiveActions.filterIsInstance<GameTurnActionData>().first()
        if (turnActionData.currentPlayerIndex == TODO()) {
            takeTurn()
        }
    }

    private fun getPOVAfterDraw(drawActionData: GameDrawActionData): HanabLivePlayerPOVAdapter {
        return game.getUpdatedWithDrawAction(drawActionData)
    }

    private fun getPOVAfterPlay(playActionData: GamePlayActionData, isStrike: Boolean): HanabLivePlayerPOVAdapter {
        return game.getUpdatedWithPlayAction(playActionData, isStrike, commonState.conventionSet)
    }

    private fun getPOVAfterDiscard(discardActionData: GameDiscardActionData): HanabLivePlayerPOVAdapter {
        return game.getUpdatedWithDiscardAction(discardActionData, commonState.conventionSet)
    }

    private fun getPOVAfterClue(gameClueActionData: GameClueActionData): HanabLivePlayerPOVAdapter {
        return game.getUpdatedWithClueAction(gameClueActionData, commonState.conventionSet)
    }

    private fun takeTurn() {
        TODO()
    }
}
