package eelst.ilike.hanablive.bot.state

import common.metadata.VariantMetadata
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
    suitMap: Map<Int, Suit>,
    rankMap: Map<Int, Rank>,
    clueValueMap: Map<Int, Map<Int, ClueValue>>
): HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState
) {
    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
    }
}
