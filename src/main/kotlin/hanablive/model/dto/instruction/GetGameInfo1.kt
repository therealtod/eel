package eelst.ilike.hanablive.model.dto.instruction

class GetGameInfo1(private val tableId: Int): HanabLiveInstruction("getGameInfo1") {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}
