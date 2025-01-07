package eelst.ilike.hanablive.entity.dto.instruction


import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : HanabLiveGameActionData(GameActionType.TURN)
