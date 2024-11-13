package eelst.ilike.utils

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import java.lang.IllegalArgumentException

object Common {
    val yamlObjectMapper = ObjectMapper(YAMLFactory()).registerKotlinModule()

    fun getResourceFileContentAsString(fileName: String): String {
        return this.javaClass.classLoader.getResource(fileName)?.readText(Charsets.UTF_8)
            ?: throw IllegalArgumentException("No resource with fileName $fileName could be found in the classPath")
    }

    fun getHandSize(numberOfPlayers: Int): Int {
        return when(numberOfPlayers) {
            6 -> 3
            4,5 -> 4
            2,3 -> 5
            else -> throw IllegalStateException("Invalid number of players: ${numberOfPlayers}")
        }
    }
}