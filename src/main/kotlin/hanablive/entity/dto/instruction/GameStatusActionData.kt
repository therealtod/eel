package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : HanabLiveGameActionData(GameActionType.STATUS)
