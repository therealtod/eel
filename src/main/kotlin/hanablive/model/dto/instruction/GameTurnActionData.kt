package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : GameActionData(GameActionType.TURN)