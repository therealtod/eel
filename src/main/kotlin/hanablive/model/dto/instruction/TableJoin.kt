package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.TableId

class TableJoin(val tableId: TableId) : HanabLiveInstruction("tableJoin") {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}
