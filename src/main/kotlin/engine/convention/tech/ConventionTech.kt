package eelst.ilike.engine.convention.tech


import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata

/**
 * A technique (tech) defined by some convention system
 */
interface ConventionTech {
    /**
     * The common name of the technique
     */
    val name: String

    /**
     * Used as a tie-breaker when two or more [GameAction] can be interpreted as instance of the same [ConventionTech]
     */
    val interpretationTier: Int

    /**
     * @return all possible [GameAction]s that can be generated using this [ConventionTech]
     */
    fun getGameActions(
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Collection<GameAction>

    /**
     * @return true if the [playAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matchesPlay(
        playAction: PlayAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean

    /**
     * @return true if the [discardAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matchesDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean

    /**
     * @return true if the [clueAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matchesClue(
        clueAction: ClueAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean

    /**
     * @return the [TeamKnowledge] acquired after the given [playAction] is performed
     */
    fun getUpdatedKnowledge(
        playAction: PlayAction,
        currentKnowledge: TeamKnowledge
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after the given [discardAction] is performed
     */
    fun getUpdatedKnowledge(
        discardAction: DiscardAction,
        currentKnowledge: TeamKnowledge
    ): TeamKnowledge

    /**
     * @return the updated [TeamKnowledge] after the given [clueAction] is performed
     */
    fun getUpdatedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        currentKnowledge: TeamKnowledge
    ): TeamKnowledge

    /**
     * @return true if the player's slot at the given [slotIndex] matches the condition needed to execute
     * this [ConventionTech]
     */
    fun slotMatchesCondition(
        slotIndex: Int,
        playerMetadata: PlayerMetadata,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean
}
