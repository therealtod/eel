package eelst.ilike.hanablive.entity.dto.instruction


data class GameStrikeActionData(
    val num: Int,
    val turn: Int,
    val order: Int,
) : HanabLiveGameActionData(GameActionType.STRIKE)
