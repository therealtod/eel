package eelst.ilike

import eelst.ilike.hanablive.bot.HanabLiveBot
import kotlinx.coroutines.runBlocking

fun main() {
    runBlocking {
        val bot = HanabLiveBot(
            username = "",
            password = "",
        )
        bot.run()
    }
}
