package eelst.ilike.engine.action

import eelst.ilike.game.entity.clue.GameAction

sealed class ObservedAction<T: GameAction>(val executedAction: ExecutedAction<T>)
    : PlayerAction<T>
