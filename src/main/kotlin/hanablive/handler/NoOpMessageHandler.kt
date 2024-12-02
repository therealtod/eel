package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType

data object NoOpMessageHandler: HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return true
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
    }

    override var nextHandler: HanabLiveInstructionHandler
        get() = TODO("Not yet implemented")
        set(value) {}
}