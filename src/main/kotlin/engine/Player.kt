package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

abstract class Player(
    val playerId: PlayerId,
    val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val hand: InterpretedHand,
)

