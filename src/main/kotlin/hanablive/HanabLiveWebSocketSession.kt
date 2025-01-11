package eelst.ilike.hanablive


import eelst.ilike.hanablive.client.HanabLiveHttpClient
import eelst.ilike.hanablive.client.HanabLiveWebSocketClient
import eelst.ilike.hanablive.exception.LoginFailedException
import eelst.ilike.hanablive.exception.UnexpectedLoginHttpResponseFormat
import io.ktor.client.plugins.websocket.*
import io.ktor.http.*
import io.ktor.websocket.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.produce
import org.apache.logging.log4j.kotlin.Logging
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

class HanabLiveWebSocketSession(val username: String, val password: String) : CoroutineScope, Logging {
    private lateinit var webSocketSession: ClientWebSocketSession

    suspend fun startSession() {
        establishConnection()
    }

    private suspend fun establishConnection() {
        val serverResponse = HanabLiveHttpClient.loginBlocking(
            username = username,
            password = password,
        )
        if (serverResponse.status.isSuccess()) {
            val responseHeaders = serverResponse.headers
            val cookieHeaderValue = responseHeaders[HanabLiveConstants.COOKIE_NAME]
                ?: throw UnexpectedLoginHttpResponseFormat(
                    "No cookie with name ${HanabLiveConstants.COOKIE_NAME} is contained in the response"
                )
            webSocketSession = HanabLiveWebSocketClient.connect(cookieHeaderValue)
        } else {
            throw LoginFailedException("The attempt to login has failed")
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    fun getIncomingMessages(): ReceiveChannel<String> = produce {
        for (frame in webSocketSession.incoming) {
            when (frame) {
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
