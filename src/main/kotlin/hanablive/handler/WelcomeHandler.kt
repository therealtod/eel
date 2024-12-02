package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType

data object WelcomeHandler : HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.WELCOME
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = TableListHandler
}
