package eelst.ilike.hanablive.client

import eelst.ilike.hanablive.HanabLiveConstants
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.websocket.*
import io.ktor.serialization.kotlinx.*
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json

object HanabLiveWebSocketClient {
    private val webSocketClient = HttpClient(CIO) {
        install(WebSockets) {
            contentConverter = KotlinxWebsocketSerializationConverter(Json)
        }
        defaultRequest {
            host = HanabLiveConstants.HOSTNAME
            port = HanabLiveConstants.PORT
        }
    }

    suspend fun connect() {

    }

    suspend fun sendData() {
        webSocketClient.webSocket(
            path = HanabLiveConstants.WEBSOCKET_PATH
        ) {
            sendSerialized("")
        }
    }

    suspend fun receiveData() {
        webSocketClient.webSocket(
            path = HanabLiveConstants.WEBSOCKET_PATH
        ) {
            val receivedData = receiveDeserialized<String>()
        }
    }

    fun connectBlocking() {
        return runBlocking { connect() }
    }

    fun sendDataBlocking() {
        return runBlocking {
            sendData()
        }
    }
}
