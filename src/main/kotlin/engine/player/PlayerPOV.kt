package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard


interface PlayerPOV {
    val globallyAvailableInfo: GloballyAvailableInfo

    fun getOwnPlayerId(): PlayerId
    fun getOwnKnownCards(): List<HanabiCard>
    fun canSee(teammatePlayerId: PlayerId, slotIndex: Int): Boolean
    fun isSlotKnown(slotIndex: Int): Boolean
    fun getPersonalKnowledge(): PlayerPersonalKnowledge
    fun getOwnKnownPlayableSlots(): Set<Slot>
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun forEachVisibleTeammate(action: (teammate: VisibleTeammate) -> Unit)
    fun getHand(playerId: PlayerId): Hand
    fun getTeammate(teammatePlayerId: PlayerId): Teammate
    fun getTeammates(): Set<Teammate>
    fun getVisibleTeammates(): Set<VisibleTeammate>
    fun getSeatsGapFrom(teammate: Teammate): Int
    fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction>
    fun asTeammateOf(teammatePlayerId: PlayerId): Teammate
    fun getOwnSlotPossibleIdentities(slotIndex: Int): Set<HanabiCard>
    fun getOwnSlotEmpathy(slotIndex: Int): Set<HanabiCard>
    fun getVisibleCards(): List<HanabiCard>
}
