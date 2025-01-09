package eelst.ilike.hanablive.bot.state


import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.bot.HanabLiveBot

/**
 * The state in which the bot is set when initialized
 */
internal class InitialState(
    bot: HanabLiveBot,
) : HanabLiveBotState(
    bot = bot,
    lobbyState = LobbyState(),
)
