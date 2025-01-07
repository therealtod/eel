package eelst.ilike.hanablive.instruction.handler

import eelst.ilike.common.Utils
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import org.apache.logging.log4j.kotlin.Logging

sealed class BaseHanabLiveInstructionHandler(
    private val nextHandler: HanabLiveInstructionHandler
) : HanabLiveInstructionHandler, Logging {
    override suspend fun handle(
        instructionType: HanabLiveInstructionType,
        instructionPayload: String,
        bot: DefaultHanabLiveBot
    ) {
        return if (supports(instructionType)) {
            doHandle(instructionPayload, bot)
        } else {
            nextHandler.handle(instructionType, instructionPayload, bot)
        }
    }

    protected abstract suspend fun doHandle(messagePayload: String, bot: DefaultHanabLiveBot)

    companion object {
        @JvmStatic
        protected val mapper = Utils.jsonObjectMapper
    }
}
