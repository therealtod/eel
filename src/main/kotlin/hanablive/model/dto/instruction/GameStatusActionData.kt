package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameStatusActionData(
    val clues: Int,
    val score: Int,
    val maxScore: Int,
) : GameActionData(GameActionType.STATUS)
