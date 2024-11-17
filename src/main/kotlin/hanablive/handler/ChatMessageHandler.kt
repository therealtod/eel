package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.client.HanabLiveWebSocketClient
import eelst.ilike.hanablive.model.dto.MessageType
import eelst.ilike.utils.Configuration
import kotlinx.coroutines.runBlocking

object ChatMessageHandler: MessageHandler(MessageType.CHAT) {
    override fun doHandle(messagePayload: String) {
        val tokens = messagePayload.split(' ')
        if (tokens.size > 1) {
            val firstToken = tokens.first()
            if (firstToken == Configuration.CHAT_MESSAGE_PREFIX) {

            }
        }
    }

    override var nextHandler: MessageHandler
        get() = TODO("Not yet implemented")
        set(value) {}
}