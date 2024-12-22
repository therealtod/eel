package eelst.ilike.hanablive.bot.state

import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.adapter.HanabLivePlayerPOVAdapter
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.*

class PlayingState(
    bot: HanabLiveBot,
    commonState: CommonState,
    private val tableId: TableId,
    private var gamePOV: HanabLivePlayerPOVAdapter,
): HanabLiveBotState(
    bot =  bot,
    commonState = commonState,
) {
    private val hanabLiveActions = mutableListOf<HanabLiveGameActionData>()

    override suspend fun onGameAction(gameAction: HanabLiveGameAction) {
        require(gameAction.tableID == tableId) {
            "Received a an action: $gameAction not related to the table I'm currently playing at: $tableId"
        }
        hanabLiveActions.add(gameAction.action)
        val turnActionReceived = hanabLiveActions.any {
            it.type == GameActionType.TURN
        }
        if (turnActionReceived) {
            handleAction(hanabLiveActions)
            hanabLiveActions.clear()
        }
    }

    private fun handleAction(hanabLiveGameActions: List<HanabLiveGameActionData>) {
        require(hanabLiveGameActions.count { HanabLiveConstants.PLAYER_ACTIONS.contains(it.type) } == 1) {
            "Only one player action should be included in this bundle: $hanabLiveGameActions"
        }
        val playerAction = hanabLiveGameActions.first { HanabLiveConstants.PLAYER_ACTIONS.contains(it.type) }
        val isStrike = hanabLiveGameActions.any { it.type == GameActionType.STRIKE }
        gamePOV = when (playerAction.type) {
            GameActionType.DRAW -> getPOVAfterDraw(playerAction as GameDrawActionData)
            GameActionType.PLAY -> getPOVAfterPlay(playerAction as GamePlayActionData, isStrike)
            GameActionType.DISCARD -> getPOVAfterDiscard(playerAction as GameDiscardActionData)
            GameActionType.CLUE -> getPOVAfterClue(playerAction as GameClueActionData)
            else -> gamePOV
        }
        val turnActionData = hanabLiveActions.filterIsInstance<GameTurnActionData>().first()
        if (turnActionData.currentPlayerIndex == gamePOV.getAsPlayer().playerIndex) {
            takeTurn()
        }
    }

    private fun getPOVAfterDraw(drawActionData: GameDrawActionData): HanabLivePlayerPOVAdapter {
        return gamePOV.getUpdatedWithDrawAction(drawActionData)
    }

    private fun getPOVAfterPlay(playActionData: GamePlayActionData, isStrike: Boolean): HanabLivePlayerPOVAdapter {
        return gamePOV.getUpdatedWithPlayAction(playActionData, isStrike, commonState.conventionSet)
    }

    private fun getPOVAfterDiscard(discardActionData: GameDiscardActionData): HanabLivePlayerPOVAdapter {
        return gamePOV.getUpdatedWithDiscardAction(discardActionData, commonState.conventionSet)
    }

    private fun getPOVAfterClue(gameClueActionData: GameClueActionData): HanabLivePlayerPOVAdapter {
        return gamePOV.getUpdatedWithClueAction(gameClueActionData, commonState.conventionSet)
    }

    private fun takeTurn() {
        TODO()
    }
}
