package eelst.ilike.hanablive.handler.request

import eelst.ilike.hanablive.model.dto.MessageType

sealed class RequestHandler(private val supportedRequestType: RequestType) {
    abstract var nextHandler: RequestHandler

    open fun handle(requestType: RequestType, messagePayload: String) {
        return if(requestType == supportedRequestType) {
            doHandle(messagePayload)
        } else {
            nextHandler.handle(requestType, messagePayload)
        }
    }

    protected abstract fun doHandle(messagePayload: String)
}