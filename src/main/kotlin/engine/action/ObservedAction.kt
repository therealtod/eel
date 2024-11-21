package eelst.ilike.engine.action

sealed class ObservedAction(val action: PlayerAction): PlayerAction(action.from)
