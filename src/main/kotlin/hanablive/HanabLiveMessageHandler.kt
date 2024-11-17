package eelst.ilike.hanablive

import eelst.ilike.hanablive.handler.ChatMessageHandler
import eelst.ilike.hanablive.model.dto.MessageType

object HanabLiveMessageHandler {
    fun handleMessage(message: String) {
        val messageTokens = message.split(' ', limit = 2)
        require(messageTokens.size == 2) {
            "Unexpected number fo tokens in the message: ${messageTokens.size}"
        }
        val messageTypeStringValue = messageTokens.first()
        val messageType = MessageType.fromStringValue(messageTypeStringValue)

        return ChatMessageHandler.handle(
            messageType, messageTokens.last()
        )
    }
}
