package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.model.dto.command.Table
import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.HanabLiveBot

data object TableHandler: HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.TABLE
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
        val table: Table = mapper.readValue(messagePayload)
        bot.putTable(table)
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = InitHandler
}