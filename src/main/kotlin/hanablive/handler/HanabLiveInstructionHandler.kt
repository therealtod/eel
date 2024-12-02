package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.utils.Utils
import org.apache.logging.log4j.kotlin.Logging

sealed class HanabLiveInstructionHandler{
    abstract val nextHandler: HanabLiveInstructionHandler

    abstract fun supports(instructionType: HanabLiveInstructionType): Boolean

    open suspend fun handle(messageType: HanabLiveInstructionType, messagePayload: String, bot: HanabLiveBot) {
        return if(supports(messageType)) {
            doHandle(messagePayload, bot)
        } else {
            if (nextHandler !is NoOpMessageHandler) {
                nextHandler.handle(messageType, messagePayload, bot)
            } else {
                println("Unhandled message of type: $messageType\n $messagePayload")
            }
        }
    }

    protected abstract suspend fun doHandle(messagePayload: String, bot: HanabLiveBot)

    companion object: Logging {
        @JvmStatic
        protected val mapper = Utils.jsonObjectMapper
    }
}
