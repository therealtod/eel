package eelst.ilike.hanablive.bot

import eelst.ilike.bot.Bot
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.hanablive.HanabLiveWebSocketSession
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import eelst.ilike.hanablive.bot.state.InitialState
import eelst.ilike.hanablive.handler.HanabLiveInstructionHandler
import eelst.ilike.hanablive.handler.WelcomeHandler
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo1
import eelst.ilike.hanablive.model.dto.instruction.HanabLiveInstruction

class HanabLiveBot(
    private val username: String,
    private val password: String,
) : Bot {
    var state: HanabLiveBotState = InitialState(
        this,
        username = username,
        password = password,
    )
    private val webSocketSession = HanabLiveWebSocketSession(
        username = username,
        password = password,
    )

    override suspend fun run() {
        webSocketSession.startSession()
        consumeWebsocketMessages(WelcomeHandler)
    }

    override fun sendMessage(message: String) {
        TODO("Not yet implemented")
    }

    override fun sendPrivateMessage(message: String, receiverId: String) {
        TODO("Not yet implemented")
    }

    override fun sendMessageToLobby(message: String) {
        TODO("Not yet implemented")
    }

    override fun playCard(slotIndex: Int) {
        TODO("Not yet implemented")
    }

    override fun discardCard(slotIndex: Int) {
        TODO("Not yet implemented")
    }

    override fun giveClue(clue: ClueValue, receiverId: String) {
        TODO("Not yet implemented")
    }

    override suspend fun joinPlayer(playerId: PlayerId) {
        state.joinPlayer(playerId)
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        state.joinPlayer(playerId, tablePassword)
    }

    override suspend fun joinTable(tableId: Int) {
        state.joinTable(tableId)
    }

    override suspend fun joinTable(tableId: Int, password: String) {
        state.joinTable(tableId, password)
    }

    suspend fun onTableStart(tableId: Int) {
        sendHanabLiveInstruction(GetGameInfo1(tableId))
    }

    suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        state.onGameInitDataReceived(gameInitData)
    }

    override fun leaveTable() {
        TODO("Not yet implemented")
    }

    suspend fun sendHanabLiveInstruction(instruction: HanabLiveInstruction) {
        webSocketSession.sendMessage(instruction.asWebSocketMessage())
    }

    suspend fun setTables(tables: Collection<Table>) {
        state.setTables(tables)
    }

    suspend fun putTable(table: Table) {
        state.putTable(table)
    }

    private suspend fun consumeWebsocketMessages(handler: HanabLiveInstructionHandler) {
        for (message in webSocketSession.getIncomingMessages()) {
            val tokens = message.split(' ', limit = 2)
            require(tokens.size > 1) {
                "The websocket message contains less that 2 tokens"
            }
            val messageTypeToken = tokens.first()
            if (HanabLiveInstructionType.entries.none { it.stringValue == messageTypeToken }) {
                continue
            }
            val payload = tokens.last()
            val messageType = HanabLiveInstructionType.fromStringValue(messageTypeToken)
            handler.handle(messageType, payload, this)
        }
    }
}
