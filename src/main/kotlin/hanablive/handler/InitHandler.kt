package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.model.dto.command.GameInitData
import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.HanabLiveBot

data object InitHandler: HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.INIT
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
        val gameInitData: GameInitData = mapper.readValue(messagePayload)
        bot.onGameInitDataReceived(gameInitData)
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = ChatMessageHandler
}