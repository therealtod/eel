package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.slot.Slot

interface TeamKnowledge {
    fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge

    /**
     * @return the updated [TeamKnowledge] after a new card is drawn
     */
    fun getAfter(drawAction: DrawAction, newSlot: Slot): GameState

    /**
     * @return the updated [TeamKnowledge] after a player plays [playedCard]
     */
    fun getAfter(
        playAction: PlayAction,
        playedCard: HanabiCard,
        isStrike: Boolean,
        conventionSet: ConventionSet
    ): GameState

    /**
     * @return the updated [TeamKnowledge] after a player discards [discardedCard]
     */
    fun getAfter(
        discardAction: DiscardAction,
        discardedCard: HanabiCard
    ): GameState

    /**
     * @return the updated [TeamKnowledge] after a player gives a clue
     */
    fun getAfter(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>
    ): GameState
}
