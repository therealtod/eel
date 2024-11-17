package eelst.ilike.hanablive

import eelst.ilike.hanablive.client.HanabLiveHttpClient
import eelst.ilike.hanablive.client.HanabLiveWebSocketClient
import eelst.ilike.utils.Configuration

object HanabLiveConnector {
    fun establishConnection() {
        val serverResponse = HanabLiveHttpClient.loginBlocking(
            username = "",
            password = "",
        )

        val responseHeaders = serverResponse.headers
        val cookieHeader = responseHeaders[HanabLiveConstants.COOKIE_NAME]
            ?: throw IllegalStateException()
        HanabLiveWebSocketClient.connectBlocking()
    }
}
