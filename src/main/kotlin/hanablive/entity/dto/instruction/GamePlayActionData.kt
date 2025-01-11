package eelst.ilike.hanablive.entity.dto.instruction


data class GamePlayActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int
) : HanabLiveGameActionData(GameActionType.PLAY)
