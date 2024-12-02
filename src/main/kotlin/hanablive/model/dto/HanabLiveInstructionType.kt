package eelst.ilike.hanablive.model.dto

enum class HanabLiveInstructionType(val stringValue: String) {
    CHAT("chat"),
    CHAT_LIST("chatList"),
    CHAT_TYPING("chatTyping"),
    GAME_ACTION("gameAction"),
    GAME_ACTION_LIST("gameActionList"),
    GAME_HISTORY("gameHistory"),
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
    USER_LEFT("userLeft"),
    USER_LIST("userList"),
    USER_INACTIVE("userInactive"),
    USER_GONE("userGone"),
    WARNING("warning"),
    WELCOME("welcome");

    companion object {
        fun fromStringValue(stringValue: String): HanabLiveInstructionType {
            return entries.find { it.stringValue == stringValue }
                ?: throw IllegalArgumentException("Unrecognized message type $stringValue")
        }
    }
}
