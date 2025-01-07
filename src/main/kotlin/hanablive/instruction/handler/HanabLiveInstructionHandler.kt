package eelst.ilike.hanablive.instruction.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType

interface HanabLiveInstructionHandler {
    /**
     * Tells whether the handler is compatible with the received instruction
     */
    fun supports(instructionType: HanabLiveInstructionType): Boolean

    /**
     * Handles the given instruction
     */
    suspend fun handle(instructionType: HanabLiveInstructionType, instructionPayload: String, bot: DefaultHanabLiveBot)
}
