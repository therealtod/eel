package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.Game
import eelst.ilike.game.GameData
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard

interface GameFromPlayerPOV: Game {
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
    fun getPlayerPOV(playerId: PlayerId): GameFromPlayerPOV
    fun getAsPlayer(): Teammate
    fun getAfter(drawAction: DrawAction, newSlot: Slot): GameFromPlayerPOV
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean, conventionSet: ConventionSet): GameFromPlayerPOV
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard, conventionSet: ConventionSet): GameFromPlayerPOV
    fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, conventionSet: ConventionSet): GameFromPlayerPOV
}
