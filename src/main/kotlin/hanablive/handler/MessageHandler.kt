package eelst.ilike.hanablive.handler

import eelst.ilike.hanablive.model.dto.MessageType

sealed class MessageHandler(private val supportedMessageType: MessageType) {
    abstract var nextHandler: MessageHandler

    open fun handle(messageType: MessageType, messagePayload: String) {
        return if(messageType == supportedMessageType) {
            doHandle(messagePayload)
        } else {
            nextHandler.handle(messageType, messagePayload)
        }
    }

    protected abstract fun doHandle(messagePayload: String)
}
