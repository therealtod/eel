package eelst.ilike.hanablive.instruction.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.dto.HanabLiveInstructionType
import hanablive.entity.dto.ChatMessage
import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.command.handler.CommandType
import eelst.ilike.hanablive.bot.command.handler.UserCommandHandler

/**
 * Handles any message sent on the hanab live chat. Being in the lobby, in a room or a private message.
 *
 * If the message is recognized as a user command, it's dispatched to provided [userCommandHandler].
 */
class ChatMessageHandler(
    private val userCommandHandler: UserCommandHandler,
    nextHandler: HanabLiveInstructionHandler,
) : BaseHanabLiveInstructionHandler(nextHandler) {
    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.CHAT
    }

    override suspend fun doHandle(messagePayload: String, bot: DefaultHanabLiveBot) {
        val chatMessageMetadata: ChatMessage = mapper.readValue(messagePayload)
        val message = chatMessageMetadata.msg
        val tokens = message.split(' ')
        if (tokens.size > 1) {
            val firstToken = tokens.first()
            if (firstToken == bot.configuration.userCommandPrefix) {
                val commandType = CommandType.fromStringValue(tokens[1])
                val args = tokens.takeLast(tokens.size - 2)
                userCommandHandler.handle(
                    commandType = commandType,
                    commandArgs = args,
                    requestSender = chatMessageMetadata.who,
                    bot = bot
                )
            }
        }
    }
}
