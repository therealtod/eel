package hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType

/**
 * Message sent from client to server when the client wants to join a table
 *
 * @param tableId the id of the table to join
 * @param password the password of the table, if the table is password protected
 */
data class TableJoin(
    private val tableId: TableId,
    private val password: String = "",
) : HanabLiveInstruction(HanabLiveInstructionType.TABLE_JOIN.label) {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId, "password" to password))
    }
}
