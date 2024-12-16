package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard


interface ActivePlayer: Player {
    val globallyAvailableInfo: GloballyAvailableInfo

    fun getOwnPlayerId(): PlayerId
    fun getOwnKnownCards(): List<HanabiCard>
    fun getPersonalKnowledge(): PlayerPersonalKnowledge
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun getTeammates(): Set<EngineHandlerPlayer>
    fun forEachTeammate(action: (engineHandlerPlayer: EngineHandlerPlayer) -> Unit)
    fun getOwnHand(): Hand
    fun getTeammate(teammatePlayerId: PlayerId): EngineHandlerPlayer
    fun getSeatsGapFrom(engineHandlerPlayer: EngineHandlerPlayer): Int
    fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction>
    fun getVisibleCards(): List<HanabiCard>
    fun getPlayerPOV(playerId: PlayerId): ActivePlayer
    fun getAsPlayer(): EngineHandlerPlayer
}
