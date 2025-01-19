package eelst.ilike.engine.knowledge

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.player.PlayerMetadata

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
     * @return this [TeamKnowledge] after the slot with index [slotIndex] of the player with metadata [playerMetadata]
     * is removed from the player's hand (by playing it or discarding it
     */
    fun withoutSlot(slotIndex: Int, playerMetadata: PlayerMetadata): TeamKnowledge

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

    /**
     * @return a new [TeamKnowledge] combining the knowledge contained in this object and [other]
     */
    fun getMergedWith(other: TeamKnowledge): TeamKnowledge
}
