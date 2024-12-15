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
    fun getPersonalKnowledge(): PlayerPersonalKnowledge
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun getTeammates(): Set<Teammate>
    fun forEachTeammate(action: (teammate: Teammate) -> Unit)
    fun getOwnHand(): Hand
    fun getTeammate(teammatePlayerId: PlayerId): Teammate
    fun getSeatsGapFrom(teammate: Teammate): Int
    fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction>
    fun getVisibleCards(): List<HanabiCard>
    fun getPlayerPOV(playerId: PlayerId): PlayerPOV
    fun getAsPlayer(): Teammate
}
