package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.HanabLiveGame
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.factory.GameStateFactory

class InGameState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,
    private val globallyAvailableGameData: GloballyAvailableGameData,
) : HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState
) {
    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        val categorizedGameActionData = parser.categorizeGameActionList(gameActionListData.list)
        val initialTeamKnowledge = parser.parseInitialTeamKnowledge(categorizedGameActionData)
        val initialPlayersState = parser.parsePlayers(categorizedGameActionData)
        val initialGameState = GameStateFactory.createGameState(
            globallyAvailableGameData = globallyAvailableGameData,
            players = initialPlayersState,
            teamKnowledge = initialTeamKnowledge
        )
        game.setInitialGameState(initialGameState)

    }

    private val game = HanabLiveGame()
    private val parser = HanabLiveDataParser(globallyAvailableGameData)
}
