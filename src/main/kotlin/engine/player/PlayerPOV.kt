package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard


interface PlayerPOV {
    val globallyAvailableInfo: GloballyAvailableInfo

    fun getOwnPlayerId(): PlayerId
    fun getOwnKnownCards(): List<HanabiCard>
    fun getOwnKnownSlots(): Set<KnownSlot>
    fun getPersonalKnowledge(): PersonalKnowledge
    fun getOwnKnownPlayableSlots(): Set<Slot>
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun forEachVisibleTeammate(action: (teammate: VisibleTeammate) -> Unit)
    fun getHand(playerId: PlayerId): InterpretedHand
    fun getTeammate(teammatePlayerId: PlayerId): Teammate
    fun getTeammates(): Set<Teammate>
    fun getVisibleTeammates(): Set<VisibleTeammate>
    fun getOwnSlot(slotIndex: Int): OwnSlot
    fun getSeatsGapFrom(teammate: Teammate): Int
    fun getOwnHand(): OwnHand
    fun getHandFromTeammatePOV(teammatePlayerId: PlayerId): OwnHand
    fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction>
    fun asTeammate(): Teammate
}
