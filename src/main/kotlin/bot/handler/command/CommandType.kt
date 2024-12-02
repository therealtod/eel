package eelst.ilike.bot.handler.request


enum class CommandType(val aliases: List<String>) {
    JOIN(listOf("join")),
    JOIN_ME(listOf("joinMe")),
    LEAVE(listOf("leave")),
    SET_CONVENTION(listOf("set_convention")),

    UNKNOWN(emptyList());

    companion object {
        fun fromStringValue(stringValue: String): CommandType {
            return entries.find { it.aliases.contains(stringValue) }
                ?: return UNKNOWN
        }
    }
}
