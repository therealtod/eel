package eelst.ilike.hanablive.bot.state

import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.HanabLiveGame
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameAction

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
