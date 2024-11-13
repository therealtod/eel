package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

interface PlayerPOV {
    val playerId: PlayerId
    val hand: InterpretedHand
    val globallyAvailableInfo: GloballyAvailableInfo
    val teammates: Set<Teammate>
    fun getVisibleCards(): List<HanabiCard>
    fun getKnownPlayableSlots(): Set<Slot>
    // fun getOwnKnownCards(): Set<HanabiCard>
    fun allCardsAreKnown(cards: Set<HanabiCard>): Boolean
    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction>
    fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction>
}