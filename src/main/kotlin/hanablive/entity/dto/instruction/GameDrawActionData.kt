package eelst.ilike.hanablive.entity.dto.instruction


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : HanabLiveGameActionData(GameActionType.DRAW)
