package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suit.Suit
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData

class InGameState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,
    private val globallyAvailableGameData: GloballyAvailableGameData,
): HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState
) {
    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
    }

    private val suitMap = globallyAvailableGameData.variant.getSuits()
        .mapIndexed { index, suit ->
            Pair(index, suit)
        }.toMap()
    private val availableClueValues = globallyAvailableGameData.variant.getClueValues()
    private val colorClues = availableClueValues.filterIsInstance<Color>()
    private val colorCluesMap = colorClues.mapIndexed { index, color -> Pair(index, color) }.toMap()
    private val rankClues = availableClueValues.filterIsInstance<Rank>().associateBy { it.numericalValue }
}
