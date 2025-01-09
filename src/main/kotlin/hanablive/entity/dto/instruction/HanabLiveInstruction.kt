package hanablive.entity.dto.instruction

import com.fasterxml.jackson.databind.ObjectMapper
import eelst.ilike.common.Utils
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType

abstract class HanabLiveInstruction(private val type: HanabLiveInstructionType) {
    fun asWebSocketMessage(): String {
        return "${type.label} ${getWebSocketPayload()}"
    }

    abstract fun getWebSocketPayload(): String

    companion object {
        @JvmStatic
        protected val mapper: ObjectMapper = Utils.jsonObjectMapper
    }
}
