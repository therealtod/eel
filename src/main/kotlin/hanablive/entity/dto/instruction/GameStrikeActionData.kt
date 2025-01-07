package eelst.ilike.hanablive.entity.dto.instruction


import eelst.ilike.hanablive.model.dto.command.GameActionType

data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : HanabLiveGameActionData(GameActionType.STRIKE)
