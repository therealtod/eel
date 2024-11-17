package eelst.ilike.hanablive.client

import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.dto.LoginResponse
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.resources.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import kotlinx.coroutines.runBlocking

object HanabLiveHttpClient {
    private val httpClient = HttpClient(CIO) {
        install(Resources)
        defaultRequest {
            url {
                protocol = URLProtocol.HTTPS
                host = HanabLiveConstants.HOSTNAME
                port = HanabLiveConstants.PORT
                path(HanabLiveConstants.LOGIN_PATH)
            }
            headers {
                append(HttpHeaders.ContentType, "application/x-www-form-urlencoded")
                append(HttpHeaders.UserAgent, "ktor client")
            }
        }
    }

    suspend fun login(username: String, password: String): HttpResponse {
        return httpClient.post(LoginResponse()) {
            setBody("username=${username}&password=${password}&version=bot")
        }
    }

    fun loginBlocking(username: String, password: String): HttpResponse {
        return runBlocking { login(username, password) }
    }
}
