package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import hanablive.entity.dto.instruction.HanabLiveInstruction

data class GetGameInfo2(private val tableId: Int) : HanabLiveInstruction(HanabLiveInstructionType.GET_GAME_INFO_2) {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}
