package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.game.entity.action.GameAction

data class GameAction(
    val tableID: String,
    val action: GameAction
)
