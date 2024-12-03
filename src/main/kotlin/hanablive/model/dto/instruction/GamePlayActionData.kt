package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.hanablive.model.dto.instruction.GameActionData

data class GamePlayActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int
) : GameActionData(GameActionType.PLAY)