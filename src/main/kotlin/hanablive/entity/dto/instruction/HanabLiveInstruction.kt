package hanablive.entity.dto.instruction

import com.fasterxml.jackson.databind.ObjectMapper
import eelst.ilike.common.Utils

abstract class HanabLiveInstruction(private val label: String) {
    fun asWebSocketMessage(): String {
        return "$label ${getWebSocketPayload()}"
    }

    abstract fun getWebSocketPayload(): String

    companion object {
        @JvmStatic
        protected val mapper: ObjectMapper = Utils.jsonObjectMapper
    }
}
