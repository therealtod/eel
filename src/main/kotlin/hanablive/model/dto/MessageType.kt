package eelst.ilike.hanablive.model.dto

enum class MessageType(val stringValue: String) {
    CHAT("chat"),
    CHAT_TYPING("chatTyping"),
    GAME_ACTION("gameAction"),
    GAME_ACTION_LIST("gameActionList"),
    JOINED("joined"),
    HYPO_START("hypoStart"),
    HYPO_END("hypoEnd"),
    INIT("init"),
    LEFT("left"),
    REPLAY_INDICATOR("replayIndicator"),
    REPLAY_SEGMENT("replaySegment"),
    TABLE("table"),
    TABLE_GONE("tableGone"),
    TABLE_LIST("tableList"),
    TABLE_START("tableStart"),
    TABLE_PROGRESS("tableProgress"),
    USER("user"),
    USER_INACTIVE("userInactive"),
    USER_GONE("userGone"),
    WARNING("warning"),
    WELCOME("welcome");

    companion object {
        fun fromStringValue(stringValue: String): MessageType {
            return entries.find { it.stringValue == stringValue }
                ?: throw IllegalArgumentException("Unrecognized message type $stringValue")
        }
    }
}
