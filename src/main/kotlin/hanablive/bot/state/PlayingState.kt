package eelst.ilike.hanablive.bot.state

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.instruction.GameAction

class PlayingState(
    bot: HanabLiveBot,
    commonState: CommonState,
    playerPOV: PlayerPOV,
): HanabLiveBotState(
    bot =  bot,
    commonState = commonState,
) {
    override suspend fun onGameAction(gameAction: GameAction) {
        TODO()
    }
}
