package eelst.ilike.hanablive.client

import eelst.ilike.hanablive.HanabLiveConstants
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.logging.*
import io.ktor.client.plugins.logging.Logger
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.util.logging.*
import io.ktor.websocket.*
import kotlinx.coroutines.isActive
import kotlinx.coroutines.runBlocking
import org.slf4j.LoggerFactory
import java.nio.charset.Charset

object HanabLiveWebSocketClient {
    private val webSocketClient = HttpClient(CIO) {
        install(WebSockets)
        install(Logging) {
            logger = Logger.DEFAULT
            level = LogLevel.HEADERS
        }
        /*
        defaultRequest {
            url {
                protocol = URLProtocol.WS
                host = HanabLiveConstants.HOSTNAME
                port = HanabLiveConstants.PORT
                path(HanabLiveConstants.WEBSOCKET_PATH)
            }
            host = HanabLiveConstants.HOSTNAME
            port = HanabLiveConstants.PORT
        }

         */
    }

    suspend fun connect(sessionIdCookieValue: String): ClientWebSocketSession {
        val session = webSocketClient.webSocketSession(
            host = HanabLiveConstants.HOSTNAME,
            path = HanabLiveConstants.WEBSOCKET_PATH,
        ){
            headers {
                append(HttpHeaders.Cookie, sessionIdCookieValue)
            }
        }
        return session
    }

    suspend fun sendData() {
        webSocketClient.webSocket(
            path = HanabLiveConstants.WEBSOCKET_PATH
        ) {
            sendSerialized("")
        }
    }
}
