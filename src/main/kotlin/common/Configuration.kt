package eelst.ilike.utils

import eelst.ilike.game.entity.suite.*


object Configuration {
    val registeredSuitesMap = mapOf(
        "red" to Red,
        "yellow" to Yellow,
        "green" to Green,
        "blue" to Blue,
        "purple" to Purple
    )

    const val CHAT_MESSAGE_PREFIX = "pls"

}
