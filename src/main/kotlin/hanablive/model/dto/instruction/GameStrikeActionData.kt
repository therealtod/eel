package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : GameActionData(GameActionType.STRIKE)
