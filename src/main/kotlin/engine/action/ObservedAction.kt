package eelst.ilike.engine.action

import eelst.ilike.game.entity.action.GameAction

sealed class ObservedAction<T: GameAction>(
    val gameAction: T,
)
