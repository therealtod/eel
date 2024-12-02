package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.TableId

class PasswordProtectedTableJoin(
    private val tableId: TableId,
    private val password: String
) : HanabLiveInstruction("tableJoin") {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId, "password" to password))
    }
}
