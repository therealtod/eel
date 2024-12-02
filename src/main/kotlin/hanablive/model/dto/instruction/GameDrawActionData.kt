package eelst.ilike.hanablive.model.dto.command


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : GameActionData(GameActionType.DRAW)
