package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData
import eelst.ilike.hanablive.entity.dto.instruction.GetGameInfo2

class LoadingGameDataState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,

): HanabLiveBotState(
    bot,
    lobbyState,
) {
    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        val variantName = gameInitData.options.variantName
        val variantMetadata = bot.getVariantMetadata(variantName)
        val suitIds = variantMetadata.suits
        val suitsMetadata = bot.getSuitsMetadata(suitIds)
        val globallyAvailableGameData = HanabLiveDataParser.parseGloballyAvailableInfo(
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        val variant = globallyAvailableGameData.variant
        val newState = InGameState(
             bot = bot,
            lobbyState = lobbyState,
            globallyAvailableGameData = globallyAvailableGameData,
        )
        switchToState(newState)
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
    }
}
