package eelst.ilike.engine.strategy

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.game.Game
import eelst.ilike.game.entity.action.GameAction

data class ActionSelectorSearchNodeData(
    val action: ConventionalAction?,
    val pov: GameFromPlayerPOV,
    val evaluation: Double,
)
