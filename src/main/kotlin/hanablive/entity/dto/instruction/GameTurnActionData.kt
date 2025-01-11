package eelst.ilike.hanablive.entity.dto.instruction


data class GameTurnActionData(
    val num: Int,
    val currentPlayerIndex: Int
) : HanabLiveGameActionData(GameActionType.TURN)
