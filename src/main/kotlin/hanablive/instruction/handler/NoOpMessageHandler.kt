package eelst.ilike.hanablive.instruction.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import org.apache.logging.log4j.kotlin.Logging

/**
 * This handler should be used as the end of the chain. indicating no compatible handler has been found for the given
 * message
 */
data object NoOpMessageHandler : HanabLiveInstructionHandler, Logging {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return true
    }

    override suspend fun handle(
        instructionType: HanabLiveInstructionType,
        instructionPayload: String,
        bot: DefaultHanabLiveBot
    ) {
        logger.info(
            "Unhandled message of type: $instructionType.\n" +
                    "Payload: $instructionPayload"
        )
    }
}
