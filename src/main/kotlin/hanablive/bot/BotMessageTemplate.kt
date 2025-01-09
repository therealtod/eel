package eelst.ilike.hanablive.bot

enum class BotMessageTemplate(val template: String) {
    CANNOT_FIND_TABLE_TO_JOIN_PLAYER(
        "You don't seem to be sitting at any table so I cannot join it"
    )
}
