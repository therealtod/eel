package eelst.ilike.hanablive.instruction.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType

class WelcomeHandler(nextHandler: HanabLiveInstructionHandler) : BaseHanabLiveInstructionHandler(nextHandler) {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.WELCOME
    }

    override suspend fun doHandle(messagePayload: String, bot: DefaultHanabLiveBot) {
    }
}
