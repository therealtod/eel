package eelst.ilike.hanablive.client

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.dto.metadata.HanabLiveSuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.HanabLiveVariantMetadata
import eelst.ilike.utils.Utils
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.jackson.*

object MetadataClient {
    private val httpClient = HttpClient(CIO) {
        install(ContentNegotiation) {
            jackson()
        }
        defaultRequest {
            url {
                protocol = URLProtocol.HTTPS
                host = HanabLiveConstants.METADATA_PROVIDER_HOSTNAME
            }
        }
    }

    suspend fun getVariantsMetadata(): List<HanabLiveVariantMetadata> {
        val response = httpClient.get {
            url {
                path(HanabLiveConstants.VARIANTS_METADATA_PATH)
            }
        }
        val bodyAsString = response.body<String>()
        return Utils.jsonObjectMapper.readValue(bodyAsString)
    }

    suspend fun getSuitsMetadata(): List<HanabLiveSuiteMetadata> {
        val response = httpClient.get {
            url {
                path(HanabLiveConstants.SUITE_METADATA_PATH)
            }
        }
        val bodyAsString = response.body<String>()
        return Utils.jsonObjectMapper.readValue(bodyAsString)
    }
}
