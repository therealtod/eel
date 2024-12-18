package eelst.ilike.hanablive.model.dto.instruction

data class GetGameInfo2(private val tableId: Int) : HanabLiveInstruction("getGameInfo2") {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}
