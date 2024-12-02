package eelst.ilike.hanablive.model.dto.instruction

import com.fasterxml.jackson.databind.ObjectMapper
import eelst.ilike.utils.Utils

abstract class HanabLiveInstruction(val label: String) {
    fun asWebSocketMessage(): String {
        return "$label ${getWebSocketPayload()}"
    }

    abstract fun getWebSocketPayload(): String

    companion object {
        @JvmStatic
        protected val mapper: ObjectMapper = Utils.jsonObjectMapper
    }
}