package eelst.ilike.hanablive.model.dto.command

enum class GameActionType(val value: String) {
    CLUE("clue"),
    DISCARD("discard"),
    DRAW("draw"),
    PLAY("play"),
    STATUS("status"),
    STRIKE("strike"),
    TURN("turn");
}