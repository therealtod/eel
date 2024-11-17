package eelst.ilike.engine.action

import eelst.ilike.game.entity.action.GameAction

sealed class ObservedAction(
    open val gameAction: GameAction,
)
