package eelst.ilike.utils

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import org.slf4j.LoggerFactory

object Utils {
    val jsonObjectMapper: ObjectMapper = jacksonObjectMapper().registerModule(JavaTimeModule())
    val yamlObjectMapper: ObjectMapper = ObjectMapper(YAMLFactory())
        .registerKotlinModule()
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)

    fun getResourceFileContentAsString(fileName: String): String {
        return this.javaClass.classLoader.getResource(fileName)?.readText(Charsets.UTF_8)
            ?: throw IllegalArgumentException("No resource with fileName $fileName could be found in the classPath")
    }

    fun <T> createLoggerFor(clazz: Class<T>) = LoggerFactory.getLogger(clazz)
}