package eelst.ilike.hanablive.entity.dto.instruction


data class GameClueActionData(
    val clue: Clue,
    val giver: Int,
    val list: List<Int>,
    val target: Int,
    val turn: Int,
) : HanabLiveGameActionData(GameActionType.CLUE) {
    data class Clue(
        val type: Int,
        val value: Int,
    )
}
