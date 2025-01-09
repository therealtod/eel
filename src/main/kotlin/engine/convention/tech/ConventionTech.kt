package eelst.ilike.engine.convention.tech


import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction

/**
 * A technique (tech) defined by some convention system
 */
interface ConventionTech {
    /**
     * The common name of the technique
     */
    val name: String

    /**
     * @return all possible [GameAction] that can be generated using this technique in the given [gameState]
     */
    fun getGameActions(gameState: GameState): Set<GameAction>

    /**
     * @return true if the [playAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matches(playAction: PlayAction, gameState: GameState): Boolean

    /**
     * @return true if the [discardAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matches(discardAction: DiscardAction, gameState: GameState): Boolean

    /**
     * @return true if the [clueAction] can be interpreted as an instance of this [ConventionTech]
     */
    fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, gameState: GameState): Boolean

    /**
     * @return what [TeamKnowledge] the team has acquired after the given [playAction] has been performed at the current
     * [gameState]
     */
    fun getGeneratedKnowledge(playAction: PlayAction, gameState: GameState): TeamKnowledge

    /**
     * @return what [TeamKnowledge] the team has acquired after the given [discardAction] has been performed at the
     * current [gameState]
     */
    fun getGeneratedKnowledge(discardAction: DiscardAction, gameState: GameState): TeamKnowledge

    /**
     * @return what [TeamKnowledge] the team has acquired after the given [clueAction] has been performed at the current
     * [gameState]
     */
    fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        gameState: GameState
    ): TeamKnowledge
}
