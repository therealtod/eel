package eelst.ilike.hanablive.bot


import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.hanablive.HanabLiveWebSocketSession
import eelst.ilike.hanablive.InstructionHandlerChainInitializer
import eelst.ilike.hanablive.bot.dto.Credentials
import eelst.ilike.hanablive.bot.dto.HanabLiveBotConfiguration
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import eelst.ilike.hanablive.bot.state.InitialState
import eelst.ilike.hanablive.bot.state.LoadingGameDataState
import eelst.ilike.hanablive.bot.state.SittingInLobbyState
import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.instruction.handler.HanabLiveInstructionHandler
import hanablive.entity.dto.instruction.GetGameInfo1
import hanablive.entity.dto.instruction.HanabLiveInstruction
import hanablive.metadata.RemoteMetadataProvider

class DefaultHanabLiveBot(
    val configuration: HanabLiveBotConfiguration,
    credentialsDTO: Credentials,
) : HanabLiveBot {
    var state: HanabLiveBotState = InitialState(
        bot = this
    )
    
    override suspend fun joinPlayer(playerId: PlayerId) {
        state.joinPlayer(playerId)
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        state.joinPlayer(playerId, tablePassword)
    }

    override suspend fun onTableStart(tableId: TableId) {
        sendHanabLiveInstruction(GetGameInfo1(tableId))
        val newState = LoadingGameDataState(
            bot = this,
            lobbyState = state.lobbyState,
        )
        switchToState(newState)
    }

    override suspend fun sendHanabLiveInstruction(instruction: HanabLiveInstruction) {
        webSocketSession.sendMessage(instruction.asWebSocketMessage())
    }

    override fun switchToState(newState: HanabLiveBotState) {
        state = newState
    }

    override suspend fun leaveTable() {
        state.leaveTable()
    }

    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        return state.onGameActionListReceived(gameActionListData)
    }

    override suspend fun getVariantMetadata(variantName: String): VariantMetadata {
        return metadataProvider.getVariantMetadata(variantName)
    }

    override suspend fun getSuitsMetadata(suitIds: Collection<SuitId>): Map<SuitId, SuitMetadata> {
        return metadataProvider.getSuitsMetadata(suitIds)
    }

    private val webSocketSession = HanabLiveWebSocketSession(
        username = credentialsDTO.username,
        password = credentialsDTO.password,
    )

    private suspend fun consumeWebsocketMessages(handler: HanabLiveInstructionHandler) {
        for (message in webSocketSession.getIncomingMessages()) {
            val tokens = message.split(' ', limit = 2)
            require(tokens.size > 1) {
                "The websocket message contains less that 2 tokens"
            }
            val messageTypeToken = tokens.first()
            if (HanabLiveInstructionType.entries.none { it.label == messageTypeToken }) {
                continue
            }
            val payload = tokens.last()
            val messageType = HanabLiveInstructionType.fromStringValue(messageTypeToken)
            handler.handle(messageType, payload, this)
        }
    }

    suspend fun run() {
        webSocketSession.startSession()
        state = SittingInLobbyState(
            bot = this
        )
        consumeWebsocketMessages(InstructionHandlerChainInitializer.getInitializedChain())
    }

    private val metadataProvider = RemoteMetadataProvider
}