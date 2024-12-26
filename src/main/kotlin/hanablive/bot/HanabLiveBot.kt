package eelst.ilike.hanablive.bot

import eelst.ilike.game.entity.clue.ClueValue
import eelst.ilike.game.model.PlayerId
import eelst.ilike.hanablive.HanabLiveWebSocketSession
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import eelst.ilike.hanablive.bot.state.HanabLiveLobbyState
import eelst.ilike.hanablive.bot.state.InitialState
import eelst.ilike.hanablive.handler.HanabLiveInstructionHandler
import eelst.ilike.hanablive.handler.WelcomeHandler
import eelst.ilike.hanablive.model.TableId


class HanabLiveBot(
    username: String,
    password: String,
) {
    var state: HanabLiveBotState = InitialState(
        this,
        username = username,
        password = password,
        commonState = HanabLiveLobbyState()
    )
    private val webSocketSession = HanabLiveWebSocketSession(
        username = username,
        password = password,
    )

    suspend fun run() {
        webSocketSession.startSession()
        consumeWebsocketMessages(WelcomeHandler)
    }

    fun sendMessage(message: String) {
        TODO("Not yet implemented")
    }

    fun sendPrivateMessage(message: String, receiverId: String) {
        TODO("Not yet implemented")
    }

    fun sendMessageToLobby(message: String) {
        TODO("Not yet implemented")
    }

    fun playCard(slotIndex: Int) {
        TODO("Not yet implemented")
    }

    fun discardCard(slotIndex: Int) {
        TODO("Not yet implemented")
    }

    fun giveClue(clue: ClueValue, receiverId: String) {
        TODO("Not yet implemented")
    }

    suspend fun joinPlayer(playerId: PlayerId) {
        state.joinPlayer(playerId)
    }

    suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        state.joinPlayer(playerId, tablePassword)
    }

    suspend fun joinTable(tableId: TableId) {
        state.joinTable(tableId)
    }

    suspend fun joinTable(tableId: TableId, password: String) {
        state.joinTable(tableId, password)
    }

    suspend fun onTableStart(tableId: Int) {
        sendHanabLiveInstruction(GetGameInfo1(tableId))
    }

    suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        state.onGameInitDataReceived(gameInitData)
    }

    fun leaveTable() {
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
