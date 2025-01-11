package eelst.ilike

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.common.Utils
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.bot.dto.Credentials
import eelst.ilike.hanablive.bot.dto.HanabLiveBotConfiguration
import kotlinx.coroutines.runBlocking

fun main() {
    val credentialsFileContent = Utils.getResourceFileContentAsString("eel-credentials.yaml")
    val credentials: Credentials = Utils.yamlObjectMapper.readValue(credentialsFileContent)
    val configurationFileContent = Utils.getResourceFileContentAsString("eel-config.yaml")
    val configuration: HanabLiveBotConfiguration = Utils.yamlObjectMapper.readValue(configurationFileContent)

    runBlocking {
        val bot = DefaultHanabLiveBot(configuration, credentials)
        bot.run()
    }
}