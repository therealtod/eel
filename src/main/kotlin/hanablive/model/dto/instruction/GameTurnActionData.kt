package eelst.ilike.hanablive.model.dto.command

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : GameActionData(GameActionType.TURN)