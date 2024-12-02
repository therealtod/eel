package eelst.ilike.hanablive.client

import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata
import eelst.ilike.utils.Utils
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.jackson.*
import com.fasterxml.jackson.module.kotlin.readValue

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

    suspend fun getVariantsMetadata(): List<VariantMetadata> {
        val response = httpClient.get {
            url {
                path(HanabLiveConstants.VARIANTS_METADATA_PATH)
            }
        }
        val bodyAsString = response.body<String>()
        return Utils.jsonObjectMapper.readValue(bodyAsString)
    }

    suspend fun getSuitsMetadata(): List<SuiteMetadata> {
        val response = httpClient.get {
            url {
                path(HanabLiveConstants.SUITE_METADATA_PATH)
            }
        }
        val bodyAsString = response.body<String>()
        return Utils.jsonObjectMapper.readValue(bodyAsString)
    }
}
