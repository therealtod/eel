package eelst.ilike.bot


import eelst.ilike.game.entity.clue.ClueValue

interface Bot {
    suspend fun run()
    fun sendMessage(message: String)
    fun sendPrivateMessage(message: String, receiverId: String)
    fun sendMessageToLobby(message: String)
    fun playCard(slotIndex: Int)
    fun discardCard(slotIndex: Int)
    fun giveClue(clue: ClueValue, receiverId: String)
    fun leaveTable()
}