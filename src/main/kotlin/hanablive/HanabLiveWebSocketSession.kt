package eelst.ilike.hanablive


import eelst.ilike.hanablive.client.HanabLiveHttpClient
import eelst.ilike.hanablive.client.HanabLiveWebSocketClient
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.*
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

class HanabLiveWebSocketSession(val username: String, val password: String): CoroutineScope {
    private lateinit var webSocketSession: ClientWebSocketSession

    suspend fun startSession() {
        establishConnection()
    }

    private suspend fun establishConnection() {
        val serverResponse = HanabLiveHttpClient.loginBlocking(
            username = username,
            password = password,
        )

        val responseHeaders = serverResponse.headers
        val cookieHeaderValue = responseHeaders[HanabLiveConstants.COOKIE_NAME]
            ?: throw IllegalStateException()
            webSocketSession = HanabLiveWebSocketClient.connect(cookieHeaderValue)
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    fun getIncomingMessages(): ReceiveChannel<String> = produce {
        for (frame in webSocketSession.incoming) {
            when(frame) {
                is Frame.Text -> send(frame.readText())
                else -> {}
            }
        }
    }

    suspend fun sendMessage(message: String) {
        webSocketSession.send(message)
    }

    override val coroutineContext: CoroutineContext
        get() = EmptyCoroutineContext
}
