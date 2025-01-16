package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.slot.Slot

interface TeamKnowledge {
    /**
     * Get what the team knows about the slot with the given [slotIndex] in the hand of the player with the given
     * [playerId]
     */
    fun getSlotKnowledge(playerId: PlayerId, slotIndex: Int)

    /**
     * @return the updated [TeamKnowledge] after a new (unspecified) card is drawn and added to the player's slot 1
     */
    fun getAfter(drawAction: DrawAction): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after the specified [card] is drawn and added to the player's slot 1
     */
    fun getAfter(drawAction: DrawAction, card: HanabiCard): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player plays an unspecified card under the agreed [conventionSet]
     */
    fun getAfter(
        playAction: PlayAction,
        conventionSet: ConventionSet
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player plays [playedCard] under the agreed [conventionSet]
     */
    fun getAfter(
        playAction: PlayAction,
        playedCard: HanabiCard,
        conventionSet: ConventionSet
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player discards an unspecifiedcard  under the agreed [conventionSet]
     */
    fun getAfter(
        discardAction: DiscardAction,
        conventionSet: ConventionSet
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player discards [discardedCard] under the agreed [conventionSet]
     */
    fun getAfter(
        discardAction: DiscardAction,
        discardedCard: HanabiCard,
        conventionSet: ConventionSet
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player gives a clue under the agreed [conventionSet]
     */
    fun getAfter(
        clueAction: ClueAction,
        touchedSlotsIndexes: Collection<Int>,
        conventionSet: ConventionSet,
    ): TeamKnowledge
}
