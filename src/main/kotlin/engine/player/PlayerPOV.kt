package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class PlayerPOV(
    val globallyAvailableInfo: GloballyAvailableInfo,
    val teammates: Set<Teammate>,
    val hand: InterpretedHand,
) {
    abstract fun getOwnKnownPlayableSlots(): Set<Slot>

    abstract fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean

    abstract fun getOwnKnownSlots(): Set<Slot>

    abstract fun getOwnKnownCards(): List<HanabiCard>
}
