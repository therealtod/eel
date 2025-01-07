package hanablive.bot.state

import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import eelst.ilike.hanablive.entity.TableId

/**
 * In this state the bot is supposed to have joined a table as a player
 */
class TableJoinedAsPlayerState(
    bot: DefaultHanabLiveBot,
    lobbyState: LobbyState,
    val tableId: TableId,
) : HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState,
)
