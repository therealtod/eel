package eelst.ilike.hanablive.entity.dto.instruction

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : HanabLiveGameActionData(GameActionType.STATUS)
