package eelst.ilike.hanablive.model.dto.command

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : GameActionData(GameActionType.STATUS)
