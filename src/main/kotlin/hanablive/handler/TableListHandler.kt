package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.command.Table

data object TableListHandler: HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.TABLE_LIST
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
        val tableMetadata: List<Table> = mapper.readValue(messagePayload)
        bot.setTables(tableMetadata)
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = TableHandler
}