package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo1

class TableStartHandler : HanabLiveInstructionHandler() {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.TABLE_START
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
        val tree = mapper.readTree(messagePayload)
        val tableId = tree.get("tableID").asInt()
        bot.sendHanabLiveInstruction(GetGameInfo1(tableId))
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = ChatMessageHandler

}