package hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType

class GetGameInfo1(private val tableId: Int) : HanabLiveInstruction(HanabLiveInstructionType.GET_GAME_INFO_1) {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}