package eelst.ilike.hanablive.model.dto.command

import java.time.ZonedDateTime

data class ChatMessage(
    val msg: String,
    val who: String,
    val discord: Boolean,
    val server: Boolean,
    val datetime: ZonedDateTime,
    val room: String,
    val recipient: String,
)
