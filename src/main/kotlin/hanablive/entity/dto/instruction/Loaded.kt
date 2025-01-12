package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import hanablive.entity.dto.instruction.HanabLiveInstruction

/**
 * Instruction to be sent when all the game data sent by the server when the game starts are correctly loaded
 */
class Loaded(private val tableId: TableId): HanabLiveInstruction(HanabLiveInstructionType.LOADED) {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(mapOf("tableID" to tableId))
    }
}
