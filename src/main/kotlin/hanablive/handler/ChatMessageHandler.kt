package eelst.ilike.hanablive.handler

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.bot.handler.request.CommandType
import eelst.ilike.bot.handler.request.JoinMyTableHandler
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.dto.HanabLiveInstructionType
import eelst.ilike.hanablive.model.dto.command.ChatMessage
import eelst.ilike.utils.Configuration

data object ChatMessageHandler : HanabLiveInstructionHandler() {
    private val commandHandler = JoinMyTableHandler

    override fun supports(instructionType: HanabLiveInstructionType): Boolean {
        return instructionType == HanabLiveInstructionType.CHAT
    }

    override suspend fun doHandle(messagePayload: String, bot: HanabLiveBot) {
        val chatMessageMetadata: ChatMessage = mapper.readValue(messagePayload)
        val message = chatMessageMetadata.msg
        val tokens = message.split(' ')
        if (tokens.size > 1) {
            val firstToken = tokens.first()
            if (firstToken == Configuration.CHAT_MESSAGE_PREFIX) {
                val commandType = CommandType.fromStringValue(tokens[1])
                val args = tokens.takeLast(tokens.size - 2)
                commandHandler.handle(
                    commandType = commandType,
                    commandArgs = args,
                    requestSender = chatMessageMetadata.who,
                    bot = bot
                )
            }
        }
    }

    override val nextHandler: HanabLiveInstructionHandler
        get() = NoOpMessageHandler
}
