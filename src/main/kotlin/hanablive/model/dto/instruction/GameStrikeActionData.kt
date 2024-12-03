package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: InterpretedHand,
) : GameActionData(GameActionType.STRIKE)
