package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDrawActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
) : HanabLiveGameActionData(GameActionType.DRAW)
