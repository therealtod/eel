package eelst.ilike.hanablive.instruction.handler

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import hanablive.entity.dto.Table

class TableHandler(nextHandler: HanabLiveInstructionHandler) : BaseHanabLiveInstructionHandler(nextHandler) {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.TABLE
    }

    override suspend fun doHandle(messagePayload: String, bot: DefaultHanabLiveBot) {
        val table: Table = mapper.readValue(messagePayload)
        bot.state.putTable(table)
    }
}