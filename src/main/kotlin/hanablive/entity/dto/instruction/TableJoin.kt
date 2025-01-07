package hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId

/**
 * Message sent from client to server when the client wants to join a table
 *
 * @param tableId the id of the table to join
 * @param password the password of the table, if the table is password protected
 */
class TableJoin(
    private val tableId: TableId,
    private val password: String = "",
) : HanabLiveInstruction("tableJoin") {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId, "password" to password))
    }
}
