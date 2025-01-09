package eelst.ilike.hanablive.entity.dto

enum class HanabLiveInstructionType(val label: String) {
    CHAT("chat"),
    CHAT_LIST("chatList"),
    CHAT_TYPING("chatTyping"),
    CHAT_PM("chatPM"),
    GAME_ACTION("gameAction"),
    GAME_ACTION_LIST("gameActionList"),
    GAME_HISTORY("gameHistory"),
    GET_GAME_INFO_1("getGameInfo1"),
    GET_GAME_INFO_2("getGameInfo2"),
    JOINED("joined"),
    HYPO_START("hypoStart"),
    HYPO_END("hypoEnd"),
    INIT("init"),
    LEFT("left"),
    REPLAY_INDICATOR("replayIndicator"),
    REPLAY_SEGMENT("replaySegment"),
    TABLE("table"),
    TABLE_GONE("tableGone"),
    TABLE_JOIN("tableJoin"),
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
            return entries.find { it.label == stringValue }
                ?: throw IllegalArgumentException("Unrecognized message type $stringValue")
        }
    }
}
