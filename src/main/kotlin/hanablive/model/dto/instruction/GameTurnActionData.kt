package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : GameActionData(GameActionType.TURN)