package eelst.ilike.bot

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.hanablive.model.TableId

interface Bot {
    suspend fun run()
    fun sendMessage(message: String)
    fun sendPrivateMessage(message: String, receiverId: String)
    fun sendMessageToLobby(message: String)
    fun playCard(slotIndex: Int)
    fun discardCard(slotIndex: Int)
    fun giveClue(clue: ClueValue, receiverId: String)
    suspend fun joinTable(tableId: TableId)
    suspend fun joinTable(tableId: TableId, password: String)
    suspend fun joinPlayer(playerId: PlayerId)
    suspend fun joinPlayer(playerId: PlayerId, tablePassword: String)
    fun leaveTable()
}