package eelst.ilike.engine.strategy

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GameState

data class ActionSelectorSearchNodeData(
    val action: ConventionalAction?,
    val gameState: GameState,
    val evaluation: Double,
)
