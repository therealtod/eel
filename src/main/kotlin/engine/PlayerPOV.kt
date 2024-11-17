package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class PlayerPOV(
    val globallyAvailableInfo: GloballyAvailableInfo,
    val teammates: Set<Teammate>,
    val hand: InterpretedHand,
) {
    abstract fun getOwnFullEmpathyCards(): List<HanabiCard>

    abstract fun getOwnKnownPlayableSlots(): Set<Slot>

    abstract fun allCardsAreKnown(cards: Set<HanabiCard>): Boolean

    abstract fun getOwnKnownSlots(): Set<Slot>

    fun getVisibleCards(): List<HanabiCard> {
        return globallyAvailableInfo.cardsOnStacks +
                globallyAvailableInfo.trashPile.cards +
                getOwnFullEmpathyCards() +
                teammates.flatMap { teammate-> teammate.hand.getCards() }
    }
}
