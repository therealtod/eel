package eelst.ilike.hanablive.bot.state

import eelst.ilike.common.model.metadata.LocalMirrorMetadataProvider
import eelst.ilike.game.*
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo2

class GameStartingState(
    bot: HanabLiveBot,
    commonState: CommonState,
) : HanabLiveBotState(
    bot = bot,
    commonState = commonState
) {
    private lateinit var botPlayerGloballyAvailableInfo: GloballyAvailablePlayerInfo
    private lateinit var game: Game
    
    private val metadataProvider = LocalMirrorMetadataProvider

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        val variantMetadata = metadataProvider.getVariantMetadata(gameInitData.options.variantName)
        val suitsMetadata = metadataProvider.getSuitsMetadata(variantMetadata.suits)
        val botPlayerIndex = gameInitData.ourPlayerIndex
        game = HanabLiveDataParser.parseGloballyAvailableInfo(
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        botPlayerGloballyAvailableInfo = game.getPlayerInfo(botPlayerIndex)
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
        val newState = GameInitDataReceivedState(
            bot = bot,
            commonState = commonState,
            botPlayerId = botPlayerGloballyAvailableInfo.playerId,
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            game = game,
        )
        bot.state = newState
    }
}
