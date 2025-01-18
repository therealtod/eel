package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction

interface TeamKnowledge {
    /**
     * Get what the team knows about the slot with the given [slotIndex] in the hand of the player with the given
     * [playerIndex]
     */
    fun getSlotKnowledge(playerIndex: Int, slotIndex: Int): SlotKnowledge

    /**
     * @return the updated [TeamKnowledge] after a new (unspecified) card is drawn and added to the player's slot 1
     */
    fun getAfterDraw(drawAction: DrawAction, globallyAvailableGameData: GloballyAvailableGameData): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after the specified [card] is drawn and added to the player's slot 1
     */
    fun getAfterDrawing(
        drawAction: DrawAction,
        card: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
        ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player plays an unspecified card
     */
    fun getAfterPlay(
        playAction: PlayAction,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player plays [playedCard]
     */
    fun getAfterPlaying(
        playAction: PlayAction,
        playedCard: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player discards an unspecified card
     */
    fun getAfterDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player discards [discardedCard]
     */
    fun getAfterDiscarding(
        discardAction: DiscardAction,
        discardedCard: HanabiCard,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after a player gives a clue
     */
    fun getAfterClueGiven(
        clueAction: ClueAction,
        touchedSlotsIndexes: Collection<Int>,
        globallyAvailableGameData: GloballyAvailableGameData,
    ): TeamKnowledge

    /**
     * @return all the cards which the player with the given [playerIndex] can  see in the hands of each players
     * (including the own hand)
     */
    fun getCardsVisibleOnHands(playerIndex: Int): Collection<HanabiCard>
}
