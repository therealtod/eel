package eelst.ilike.hanablive.bot.state

import eelst.ilike.common.model.metadata.LocalMirrorMetadataProvider
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.PersonalSlotKnowledgeImpl
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.game.*
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.SimpleHand
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.RemoteMetadataProvider
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo2
import kotlinx.coroutines.runBlocking

class GameStartingState(
    bot: HanabLiveBot,
    commonState: CommonState,
) : HanabLiveBotState(
    bot = bot,
    commonState = commonState
) {
    private lateinit var botPlayerGloballyAvailableInfo: GloballyAvailablePlayerInfo
    private lateinit var globallyAvailableInfo: GloballyAvailableInfo
    
    private val metadataProvider = LocalMirrorMetadataProvider

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        val variantMetadata = metadataProvider.getVariantMetadata(gameInitData.options.variantName)
        val suitsMetadata = metadataProvider.getSuitsMetadata(variantMetadata.suits)
        val botPlayerIndex = gameInitData.ourPlayerIndex
        globallyAvailableInfo = HanabLiveDataParser.parseGloballyAvailableInfo(
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        botPlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(botPlayerIndex)
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
        val newState = GameInitDataReceivedState(
            bot = bot,
            commonState = commonState,
            botPlayerId = botPlayerGloballyAvailableInfo.playerId,
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            globallyAvailableInfo = globallyAvailableInfo,
        )
        bot.state = newState
    }
}
