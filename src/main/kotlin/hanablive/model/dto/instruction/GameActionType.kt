package eelst.ilike.hanablive.model.dto.command

import com.fasterxml.jackson.annotation.JsonValue

enum class GameActionType(val value: String) {
    CLUE("clue"),
    DISCARD("discard"),
    DRAW("draw"),
    PLAY("play"),
    STATUS("status"),
    STRIKE("strike"),
    TURN("turn");

    @JsonValue
    fun getJsonValue(): String {
        return value
    }
}
