package eelst.ilike.hanablive.bot.state

import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.hanablive.bot.HanabLiveBot

class PlayingState(
    bot: HanabLiveBot,
    commonState: CommonState,
    activePlayer: ActivePlayer,
): HanabLiveBotState(
    bot =  bot,
    commonState = commonState,
)
