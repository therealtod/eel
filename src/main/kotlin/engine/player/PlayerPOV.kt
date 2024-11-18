package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class PlayerPOV(
    val playerId: PlayerId,
    val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val teammates: Set<Teammate>,
    val hand: OwnHand,
) {
    abstract fun getOwnKnownPlayableSlots(): Set<Slot>

    abstract fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean

    abstract fun getOwnKnownSlots(): Set<Slot>

    abstract fun getOwnKnownCards(): List<HanabiCard>

    abstract fun getActions(conventionSet: ConventionSet): Set<ConventionalAction>

    abstract fun knows(slotIndex: Int): Boolean

    abstract fun getSlotFromPlayerPOV(slotIndex: Int): OwnSlot
}
