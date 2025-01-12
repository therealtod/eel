package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.HanabLiveGame
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.entity.dto.instruction.GameActionType
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameAction
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameActionData
import eelst.ilike.hanablive.factory.GameStateFactory

class InGameState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,
    private val game: HanabLiveGame,
) : HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState
) {
    override suspend fun onGameAction(gameAction: HanabLiveGameAction) {
        super.onGameAction(gameAction)
    }

    private fun takeTurn() {
        TODO("If it's the bot's turn, choose and act")
    }
}
