package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.dto.command.GameActionType


data class GameDiscardActionData(
    val playerIndex: Int,
    val order: Int,
    val suitIndex: Int,
    val rank: Int,
    val failed: Boolean,
) : GameActionData(GameActionType.DISCARD)
