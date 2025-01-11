package eelst.ilike.hanablive.entity.dto.instruction

import com.fasterxml.jackson.annotation.JsonValue

enum class GameActionType(private val jsonAlias: String, val isTurnDefiningAction: Boolean) {
    CLUE("clue", true),
    DISCARD("discard", true),
    DRAW("draw", false),
    PLAY("play", true),
    STATUS("status", false),
    STRIKE("strike", false),
    TURN("turn", false);

    @JsonValue
    fun getJsonValue(): String {
        return jsonAlias
    }
}
