package eelst.ilike.engine.action

import eelst.ilike.game.PlayerId

sealed class GameAction(open val from: PlayerId)
