package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import hanablive.entity.dto.instruction.HanabLiveInstruction

data class ChatPM(
    val message: String,
    val recipient: PlayerId,
    val room: String,
) : HanabLiveInstruction(HanabLiveInstructionType.CHAT_PM) {
    override fun getWebSocketPayload(): String {
        return mapper.writeValueAsString(
            mapOf(
                "msg" to message,
                "recipient" to recipient,
                "room" to room,
            )
        )
    }
}
