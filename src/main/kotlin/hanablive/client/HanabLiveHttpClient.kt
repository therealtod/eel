package eelst.ilike.hanablive.client

import eelst.ilike.hanablive.HanabLiveConstants
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.resources.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import kotlinx.coroutines.runBlocking

object HanabLiveHttpClient {
    private val httpClient = HttpClient(CIO) {
        defaultRequest {
            url {
                protocol = URLProtocol.HTTPS
                host = HanabLiveConstants.HOSTNAME
                port = HanabLiveConstants.HANAB_LIVE_HTTP_PORT
                path(HanabLiveConstants.LOGIN_PATH)
            }
            headers {
                append(HttpHeaders.ContentType, "application/x-www-form-urlencoded")
                append(HttpHeaders.UserAgent, "ktor client")
            }
        }
    }

    suspend fun login(username: String, password: String): HttpResponse {
        val response = httpClient.post {
            setBody("username=${username}&password=${password}&version=bot")
        }
        return response
    }

    fun loginBlocking(username: String, password: String): HttpResponse {
        return runBlocking { login(username, password) }
    }
}
