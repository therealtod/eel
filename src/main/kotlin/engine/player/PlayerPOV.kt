package eelst.ilike.engine.player

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.Game
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

interface PlayerPOV: Player {
    val game: Game

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
    fun getAfter(
        gameAction: ObservedAction,
        game: Game,
        techs: Collection<ConventionTech>,
    ): PlayerPOV
}
